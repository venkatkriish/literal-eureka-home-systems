use chrono::Utc;

use crate::db::AppState;
use crate::models::{
    Bill, BillStatus, BillingDate, BillingMonth, DashboardView, DomainError, Money, NewBillInput,
    NewPaymentInput, Payment, PaymentStatus, Role, StatsSummary, UpdateUserInput, User,
};
use crate::pdf;

pub fn list_users(state: &AppState) -> Vec<User> { state.users.lock().unwrap().clone() }
pub fn list_bills(state: &AppState) -> Vec<Bill> { state.bills.lock().unwrap().clone() }

pub fn search_bills(state: &AppState, user_id: Option<u32>, month: Option<String>) -> Vec<Bill> {
    let bills = state.bills.lock().unwrap();
    bills.iter()
        .filter(|bill| user_id.map_or(true, |id| bill.user_id == id))
        .filter(|bill| month.as_ref().map_or(true, |m| bill.month.to_string() == *m))
        .cloned().collect()
}

pub fn dashboard_view(state: &AppState) -> DashboardView {
    DashboardView { summary: stats_summary(state), users: list_users(state), bills: list_bills(state) }
}

pub fn stats_summary(state: &AppState) -> StatsSummary {
    let users = state.users.lock().unwrap();
    let bills = state.bills.lock().unwrap();
    let total_units = bills.iter().map(|b| b.units).sum::<f64>();
    let total_revenue = bills.iter().map(|bill| bill.units * bill.rate_per_unit.0 as f64 + bill.fixed_charge.0 as f64).sum::<f64>();
    let paid_bills = bills.iter().filter(|b| b.status == BillStatus::Paid).count();
    let pending_bills = bills.iter().filter(|b| b.status == BillStatus::Pending).count();
    let overdue_bills = bills.iter().filter(|b| b.status == BillStatus::Overdue).count();
    let average_bill = if bills.is_empty() { Money(0) } else { Money((total_revenue / bills.len() as f64).round() as i64) };
    StatsSummary { total_users: users.len(), total_bills: bills.len(), paid_bills, pending_bills, overdue_bills, total_units, total_revenue: Money(total_revenue.round() as i64), average_bill }
}

pub fn create_bill(state: &AppState, input: NewBillInput) -> Result<Bill, String> {
    let month = BillingMonth::parse(&input.month).map_err(|_| DomainError::InvalidMonth.to_string())?;
    if input.units <= 0.0 { return Err(DomainError::Validation("units must be positive".into()).to_string()); }
    let mut bills = state.bills.lock().unwrap();
    let id = bills.len() as u32 + 1;
    let rate_per_unit = Money::new((input.rate_per_unit.parse::<f64>().map_err(|_| DomainError::InvalidMoney.to_string())? * 100.0).round() as i64).map_err(|e| e.to_string())?;
    let fixed_charge = Money::new((input.fixed_charge.parse::<f64>().map_err(|_| DomainError::InvalidMoney.to_string())? * 100.0).round() as i64).map_err(|e| e.to_string())?;
    let amount_due_cents = (input.units * rate_per_unit.0 as f64).round() as i64 + fixed_charge.0;
    let amount_due = Money::new(amount_due_cents).map_err(|e| e.to_string())?;
    let bill = Bill { id, user_id: input.user_id, month, units: input.units, rate_per_unit, fixed_charge, amount_due, due_date: BillingDate::parse("2026-01-28").map_err(|_| DomainError::Validation("invalid date".into()).to_string())?, status: BillStatus::Pending, consumption_history: vec![], payments: vec![] };
    bills.push(bill.clone());
    Ok(bill)
}

pub fn record_payment(state: &AppState, bill_id: u32, input: NewPaymentInput) -> Result<Bill, String> {
    let mut bills = state.bills.lock().unwrap();
    let bill = bills.iter_mut().find(|b| b.id == bill_id).ok_or_else(|| DomainError::NotFound.to_string())?;
    bill.status = BillStatus::Paid;
    let payment = Payment { id: bill.payments.len() as u32 + 1, month: bill.month.clone(), amount: bill.amount_due, paid_at: Utc::now(), status: input.status.unwrap_or(PaymentStatus::Completed) };
    bill.payments.push(payment);
    Ok(bill.clone())
}

pub fn update_user(state: &AppState, user_id: u32, input: UpdateUserInput) -> Result<User, String> {
    let mut users = state.users.lock().unwrap();
    let user = users.iter_mut().find(|u| u.id == user_id).ok_or_else(|| DomainError::NotFound.to_string())?;
    if let Some(name) = input.name { user.name = name; }
    if let Some(role) = input.role { user.role = role; }
    Ok(user.clone())
}

pub fn bill_for_id(state: &AppState, bill_id: u32) -> Result<(Bill, User, User), String> {
    let bills = state.bills.lock().unwrap();
    let users = state.users.lock().unwrap();
    let bill = bills.iter().find(|b| b.id == bill_id).cloned().ok_or_else(|| DomainError::NotFound.to_string())?;
    let customer = users.iter().find(|u| u.id == bill.user_id).cloned().ok_or_else(|| DomainError::NotFound.to_string())?;
    let viewer = users.iter().find(|u| matches!(u.role, Role::Admin)).cloned().unwrap_or_else(|| customer.clone());
    Ok((bill, customer, viewer))
}

pub fn download_bill_pdf(state: &AppState, bill_id: u32) -> Result<Vec<u8>, String> {
    let (bill, customer, viewer) = bill_for_id(state, bill_id)?;
    Ok(pdf::render_bill_pdf(&bill, &customer, &viewer))
}
