use chrono::Utc;

use crate::{
    db::AppState,
    models::{Bill, BillStatus, BillingDate, BillingMonth, DashboardView, Money, NewBillInput, NewPaymentInput, Payment, PaymentStatus, Role, StatsSummary, UpdateUserInput},
};

pub fn list_users(state: &AppState) -> Vec<crate::models::User> {
    state.users.lock().unwrap().clone()
}

pub fn list_bills(state: &AppState) -> Vec<Bill> {
    state.bills.lock().unwrap().clone()
}

pub fn search_bills(state: &AppState, month: Option<String>, status: Option<String>, user_id: Option<u32>) -> Vec<Bill> {
    list_bills(state)
        .into_iter()
        .filter(|bill| month.as_ref().map(|m| bill.month.to_string() == *m).unwrap_or(true))
        .filter(|bill| status.as_ref().map(|s| format!("{:?}", bill.status).to_lowercase() == s.to_lowercase()).unwrap_or(true))
        .filter(|bill| user_id.map(|id| bill.user_id == id).unwrap_or(true))
        .collect()
}

pub fn dashboard_view(state: &AppState) -> DashboardView {
    DashboardView { summary: stats_summary(state), users: list_users(state), bills: list_bills(state) }
}

pub fn stats_summary(state: &AppState) -> StatsSummary {
    let users = list_users(state);
    let bills = list_bills(state);
    let total_units: u32 = bills.iter().map(|bill| bill.units).sum();
    let total_revenue_cents: i64 = bills.iter().map(|bill| bill.amount_due.as_cents()).sum();
    let paid_bills = bills.iter().filter(|bill| matches!(bill.status, BillStatus::Paid)).count();
    let pending_bills = bills.iter().filter(|bill| matches!(bill.status, BillStatus::Pending)).count();
    let average_bill = if bills.is_empty() { Money::from_cents(0).unwrap() } else { Money::from_cents((total_revenue_cents / bills.len() as i64).max(0)).unwrap() };
    StatsSummary {
        total_users: users.len(),
        total_bills: bills.len(),
        paid_bills,
        pending_bills,
        total_units,
        total_revenue: Money::from_cents(total_revenue_cents.max(0)).unwrap(),
        average_bill,
    }
}

pub fn create_bill(state: &AppState, input: NewBillInput) -> Result<Bill, String> {
    let month = BillingMonth::parse(input.month)?;
    let due_date = BillingDate::parse(input.due_date)?;
    let rate_per_unit = Money::from_rupees_str(&input.rate_per_unit)?;
    let fixed_charge = Money::from_rupees_str(&input.fixed_charge)?;
    let amount_due = Money::from_cents(input.units as i64 * rate_per_unit.as_cents() + fixed_charge.as_cents()).map_err(|e| e)?;
    let mut bills = state.bills.lock().unwrap();
    let bill = Bill { id: bills.len() as u32 + 1, user_id: input.user_id, month, units: input.units, rate_per_unit, fixed_charge, amount_due, due_date, status: BillStatus::Pending, consumption_history: vec![], payments: vec![] };
    bills.push(bill.clone());
    Ok(bill)
}

pub fn record_payment(state: &AppState, bill_id: u32, input: NewPaymentInput) -> Result<Bill, String> {
    let amount = Money::from_rupees_str(&input.amount)?;
    let mut bills = state.bills.lock().unwrap();
    let bill = bills.iter_mut().find(|bill| bill.id == bill_id).ok_or_else(|| "bill not found".to_string())?;
    let payment = Payment { id: bill.payments.len() as u32 + 1, month: bill.month.clone(), amount, paid_at: Utc::now(), status: PaymentStatus::Completed };
    bill.payments.push(payment);
    bill.status = BillStatus::Paid;
    Ok(bill.clone())
}

pub fn update_user(state: &AppState, id: u32, input: UpdateUserInput) -> Result<crate::models::User, String> {
    let mut users = state.users.lock().unwrap();
    let user = users.iter_mut().find(|user| user.id == id).ok_or_else(|| "user not found".to_string())?;
    if let Some(name) = input.name { user.name = name; }
    if let Some(email) = input.email { user.email = email; }
    if let Some(meter_number) = input.meter_number { user.meter_number = meter_number; }
    if let Some(address) = input.address { user.address = address; }
    if let Some(role) = input.role { user.role = role; }
    Ok(user.clone())
}
