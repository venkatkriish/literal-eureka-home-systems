use chrono::{NaiveDate, Utc};

use crate::{
    db::AppState,
    models::{ApiError, Bill, BillStatus, BillingDate, BillingMonth, DashboardView, Money, NewBillInput, NewPaymentInput, Payment, PaymentStatus, Role, StatsSummary, UpdateUserInput, User},
    pdf,
};

pub fn list_users(state: &AppState) -> Vec<User> {
    state.users.lock().unwrap().clone()
}

pub fn list_bills(state: &AppState) -> Vec<Bill> {
    state.bills.lock().unwrap().clone()
}

pub fn search_bills(state: &AppState, month: Option<String>, status: Option<String>, user_id: Option<u32>) -> Vec<Bill> {
    state
        .bills
        .lock()
        .unwrap()
        .iter()
        .filter(|bill| month.as_ref().map(|m| bill.month.0 == *m).unwrap_or(true))
        .filter(|bill| status.as_ref().map(|s| bill_status_name(&bill.status) == s.to_lowercase()).unwrap_or(true))
        .filter(|bill| user_id.map(|id| bill.user_id == id).unwrap_or(true))
        .cloned()
        .collect()
}

pub fn dashboard_view(state: &AppState) -> DashboardView {
    DashboardView { summary: stats_summary(state), users: list_users(state), bills: list_bills(state) }
}

pub fn stats_summary(state: &AppState) -> StatsSummary {
    let bills = state.bills.lock().unwrap();
    let total_bills = bills.len();
    let paid_bills = bills.iter().filter(|bill| bill.status == BillStatus::Paid).count();
    let pending_bills = bills.iter().filter(|bill| bill.status == BillStatus::Pending).count();
    let overdue_bills = bills.iter().filter(|bill| bill.status == BillStatus::Overdue).count();
    let total_units = bills.iter().map(|bill| bill.units).sum();
    let total_revenue = Money(bills.iter().map(|bill| bill.amount_due.cents()).sum());
    let average_bill = if total_bills == 0 { Money::zero() } else { Money((total_revenue.cents() as f64 / total_bills as f64).round() as i64) };
    StatsSummary { total_users: state.users.lock().unwrap().len(), total_bills, paid_bills, pending_bills, overdue_bills, total_units, total_revenue, average_bill }
}

pub fn create_bill(state: &AppState, input: NewBillInput) -> Result<Bill, String> {
    if input.units <= 0.0 { return Err("units must be positive".into()); }
    let month = BillingMonth::parse(&input.month).map_err(|e| e.to_string())?;
    let rate_per_unit = parse_money(&input.rate_per_unit)?;
    let fixed_charge = parse_money(&input.fixed_charge)?;
    let due_date = NaiveDate::parse_from_str(&input.due_date, "%Y-%m-%d").map_err(|_| "invalid due date".to_string())?;
    let amount_due = calculate_amount_due(input.units, rate_per_unit, fixed_charge)?;

    let mut bills = state.bills.lock().unwrap();
    let bill = Bill { id: next_bill_id(&bills), user_id: input.user_id, month, units: input.units, rate_per_unit, fixed_charge, amount_due, due_date: BillingDate(due_date), status: BillStatus::Pending, consumption_history: vec![], payments: vec![] };
    bills.push(bill.clone());
    Ok(bill)
}

pub fn record_payment(state: &AppState, bill_id: u32, input: NewPaymentInput) -> Result<Bill, String> {
    let amount = parse_money(&input.amount)?;
    if amount.cents() <= 0 { return Err("payment amount must be positive".into()); }
    let mut bills = state.bills.lock().unwrap();
    let bill = bills.iter_mut().find(|bill| bill.id == bill_id).ok_or_else(|| "bill not found".to_string())?;
    if bill.payments.iter().any(|payment| payment.status == PaymentStatus::Completed && payment.amount.cents() == amount.cents()) {
        return Err("duplicate payment amount already recorded".into());
    }

    let payment_status = input.status.unwrap_or(PaymentStatus::Completed);
    let payment = Payment { id: next_payment_id(&bill.payments), month: bill.month.clone(), amount, paid_at: Utc::now(), status: payment_status.clone() };
    bill.payments.push(payment);

    let completed_total: i64 = bill.payments.iter().filter(|payment| payment.status == PaymentStatus::Completed).map(|payment| payment.amount.cents()).sum();
    if completed_total > bill.amount_due.cents() {
        bill.payments.pop();
        return Err("payment would overpay the bill".into());
    }

    bill.status = match payment_status {
        PaymentStatus::Completed if completed_total == bill.amount_due.cents() => BillStatus::Paid,
        PaymentStatus::Completed if is_overdue(&bill.due_date.0) => BillStatus::Overdue,
        PaymentStatus::Completed => BillStatus::Pending,
        PaymentStatus::Failed | PaymentStatus::Refunded => {
            if is_overdue(&bill.due_date.0) && completed_total < bill.amount_due.cents() { BillStatus::Overdue } else if completed_total == bill.amount_due.cents() { BillStatus::Paid } else { BillStatus::Pending }
        }
    };

    Ok(bill.clone())
}

pub fn update_user(state: &AppState, user_id: u32, input: UpdateUserInput) -> Result<User, String> {
    let mut users = state.users.lock().unwrap();
    let user = users.iter_mut().find(|user| user.id == user_id).ok_or_else(|| "user not found".to_string())?;
    if let Some(name) = input.name { user.name = name; }
    if let Some(email) = input.email { user.email = email; }
    if let Some(meter_number) = input.meter_number { user.meter_number = meter_number; }
    if let Some(address) = input.address { user.address = address; }
    if let Some(role) = input.role { user.role = role; }
    Ok(user.clone())
}

/// Why a bill PDF can't be served.
#[derive(Debug, PartialEq, Eq)]
pub enum PdfAccessError {
    /// No `X-User-Id`, or it names no user.
    Unauthenticated,
    /// The viewer is neither the bill's customer nor an admin.
    Forbidden,
    NotFound,
}

/// The bill and its customer.
pub fn bill_for_id(state: &AppState, bill_id: u32) -> Result<(Bill, User), String> {
    let bills = state.bills.lock().unwrap();
    let users = state.users.lock().unwrap();
    let bill = bills.iter().find(|b| b.id == bill_id).cloned().ok_or_else(|| "bill not found".to_string())?;
    let customer = users.iter().find(|u| u.id == bill.user_id).cloned().ok_or_else(|| "customer not found".to_string())?;
    Ok((bill, customer))
}

/// The bill as a PDF, for its own customer or an admin only.
pub fn download_bill_pdf(state: &AppState, bill_id: u32, viewer_id: Option<u32>) -> Result<Vec<u8>, PdfAccessError> {
    let viewer = viewer_id
        .and_then(|id| state.users.lock().unwrap().iter().find(|u| u.id == id).cloned())
        .ok_or(PdfAccessError::Unauthenticated)?;
    let (bill, customer) = bill_for_id(state, bill_id).map_err(|_| PdfAccessError::NotFound)?;
    if viewer.id != customer.id && !matches!(viewer.role, Role::Admin) {
        return Err(PdfAccessError::Forbidden);
    }
    let lines = pdf::build_bill_pdf_lines(&bill, &customer, &viewer);
    Ok(pdf::render_bill_pdf_from_lines(&lines))
}

fn parse_money(value: &str) -> Result<Money, String> {
    crate::models::parse_money(value).map_err(|e| e.to_string())
}

fn calculate_amount_due(units: f64, rate_per_unit: Money, fixed_charge: Money) -> Result<Money, String> {
    let energy_charge = (units * rate_per_unit.cents() as f64).round();
    if !energy_charge.is_finite() || energy_charge < 0.0 || energy_charge > i64::MAX as f64 { return Err("bill total overflowed".into()); }
    Money::new(energy_charge as i64 + fixed_charge.cents()).map_err(|e| e.to_string())
}

fn next_bill_id(items: &[Bill]) -> u32 { items.iter().map(|bill| bill.id).max().unwrap_or(0) + 1 }
fn next_payment_id(items: &[Payment]) -> u32 { items.iter().map(|payment| payment.id).max().unwrap_or(0) + 1 }
fn bill_status_name(status: &BillStatus) -> &'static str { match status { BillStatus::Pending => "pending", BillStatus::Paid => "paid", BillStatus::Overdue => "overdue" } }
fn is_overdue(due_date: &NaiveDate) -> bool { *due_date < Utc::now().date_naive() }

pub fn to_api_error(message: String) -> (axum::http::StatusCode, axum::Json<ApiError>) {
    (axum::http::StatusCode::BAD_REQUEST, axum::Json(ApiError { message }))
}
