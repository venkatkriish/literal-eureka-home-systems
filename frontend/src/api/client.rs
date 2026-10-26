use backend::models::{Bill, BillStatus, BillingDate, BillingMonth, DashboardView, Money, StatsSummary, User};

#[derive(Clone, Debug)]
pub struct Client {
    pub base_url: String,
}

impl Client {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self { base_url: base_url.into() }
    }

    pub fn dashboard_from_seed(&self, users: Vec<User>, bills: Vec<Bill>) -> DashboardView {
        let summary = StatsSummary {
            total_users: users.len(),
            total_bills: bills.len(),
            paid_bills: bills.iter().filter(|bill| matches!(bill.status, BillStatus::Paid)).count(),
            pending_bills: bills.iter().filter(|bill| matches!(bill.status, BillStatus::Pending)).count(),
            total_units: bills.iter().map(|bill| bill.units).sum(),
            total_revenue: Money::from_cents(bills.iter().map(|bill| bill.amount_due.as_cents()).sum()).unwrap_or_else(|_| Money::from_cents(0).unwrap()),
            average_bill: Money::from_cents(if bills.is_empty() { 0 } else { bills.iter().map(|bill| bill.amount_due.as_cents()).sum::<i64>() / bills.len() as i64 }).unwrap_or_else(|_| Money::from_cents(0).unwrap()),
        };
        DashboardView { summary, users, bills }
    }
}

pub fn sample_users() -> Vec<User> {
    vec![
        User { id: 1, name: "Asha Kumar".into(), email: "asha@example.com".into(), meter_number: "MTR-1001".into(), address: "12 Lake Road".into(), role: backend::models::Role::User },
        User { id: 2, name: "Rahul Singh".into(), email: "rahul@example.com".into(), meter_number: "MTR-1002".into(), address: "44 Green Avenue".into(), role: backend::models::Role::User },
        User { id: 3, name: "Board Admin".into(), email: "admin@board.local".into(), meter_number: "ADMIN-01".into(), address: "Operations Center".into(), role: backend::models::Role::Admin },
    ]
}

pub fn sample_bills() -> Vec<Bill> {
    vec![
        Bill { id: 1, user_id: 1, month: BillingMonth::parse("2026-01").expect("valid billing month"), units: 210, rate_per_unit: Money::from_cents(650).expect("valid money"), fixed_charge: Money::from_cents(4500).expect("valid money"), amount_due: Money::from_cents(141000).expect("valid money"), due_date: BillingDate::parse("2026-01-28").expect("valid billing date"), status: BillStatus::Pending, consumption_history: vec![], payments: vec![] },
        Bill { id: 2, user_id: 1, month: BillingMonth::parse("2025-12").expect("valid billing month"), units: 195, rate_per_unit: Money::from_cents(625).expect("valid money"), fixed_charge: Money::from_cents(4500).expect("valid money"), amount_due: Money::from_cents(126375).expect("valid money"), due_date: BillingDate::parse("2025-12-28").expect("valid billing date"), status: BillStatus::Paid, consumption_history: vec![], payments: vec![] },
        Bill { id: 3, user_id: 2, month: BillingMonth::parse("2026-01").expect("valid billing month"), units: 160, rate_per_unit: Money::from_cents(590).expect("valid money"), fixed_charge: Money::from_cents(4000).expect("valid money"), amount_due: Money::from_cents(98800).expect("valid money"), due_date: BillingDate::parse("2026-01-25").expect("valid billing date"), status: BillStatus::Pending, consumption_history: vec![], payments: vec![] },
    ]
}
