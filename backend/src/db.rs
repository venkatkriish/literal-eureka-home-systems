use std::sync::{Arc, Mutex};

use chrono::{NaiveDate, TimeZone, Utc};

use crate::models::{Bill, BillStatus, BillingDate, BillingMonth, ConsumptionRecord, Money, Payment, PaymentStatus, Role, User};

#[derive(Clone)]
pub struct AppState {
    pub users: Arc<Mutex<Vec<User>>>,
    pub bills: Arc<Mutex<Vec<Bill>>>,
}

impl AppState {
    pub fn seeded() -> Self {
        let users = vec![
            User { id: 1, name: "Asha Kumar".into(), email: "asha@example.com".into(), meter_number: "MTR-1001".into(), address: "12 Lake Road".into(), role: Role::User },
            User { id: 2, name: "Rahul Singh".into(), email: "rahul@example.com".into(), meter_number: "MTR-1002".into(), address: "44 Green Avenue".into(), role: Role::User },
            User { id: 3, name: "Meera Iyer".into(), email: "meera@example.com".into(), meter_number: "MTR-1003".into(), address: "91 Hill Crest".into(), role: Role::User },
            User { id: 4, name: "Board Admin".into(), email: "admin@board.local".into(), meter_number: "ADMIN-01".into(), address: "Operations Center".into(), role: Role::Admin },
        ];

        let paid_at_december = make_timestamp(2025, 12, 20, 10, 30, 0).expect("static seed timestamp must be valid");
        let paid_at_november = make_timestamp(2025, 11, 22, 9, 0, 0).expect("static seed timestamp must be valid");

        let bills = vec![
            Bill { id: 1, user_id: 1, month: BillingMonth("2026-01".into()), units: 210.0, rate_per_unit: Money(650), fixed_charge: Money(4500), amount_due: Money(141000), due_date: BillingDate(date(2026, 1, 28).expect("static seed date must be valid")), status: BillStatus::Pending, consumption_history: vec![ConsumptionRecord { month: BillingMonth("2025-11".into()), units: 180.0, rate_per_unit: Money(600), fixed_charge: Money(4500) }, ConsumptionRecord { month: BillingMonth("2025-12".into()), units: 195.0, rate_per_unit: Money(625), fixed_charge: Money(4500) }], payments: vec![] },
            Bill { id: 2, user_id: 1, month: BillingMonth("2025-12".into()), units: 195.0, rate_per_unit: Money(625), fixed_charge: Money(4500), amount_due: Money(126375), due_date: BillingDate(date(2025, 12, 28).expect("static seed date must be valid")), status: BillStatus::Paid, consumption_history: vec![], payments: vec![Payment { id: 1, month: BillingMonth("2025-12".into()), amount: Money(126375), paid_at: paid_at_december, status: PaymentStatus::Completed }] },
            Bill { id: 3, user_id: 2, month: BillingMonth("2026-01".into()), units: 160.0, rate_per_unit: Money(590), fixed_charge: Money(4000), amount_due: Money(98800), due_date: BillingDate(date(2026, 1, 25).expect("static seed date must be valid")), status: BillStatus::Pending, consumption_history: vec![], payments: vec![] },
            Bill { id: 4, user_id: 3, month: BillingMonth("2026-01".into()), units: 240.0, rate_per_unit: Money(680), fixed_charge: Money(5000), amount_due: Money(213200), due_date: BillingDate(date(2026, 1, 24).expect("static seed date must be valid")), status: BillStatus::Overdue, consumption_history: vec![], payments: vec![] },
            Bill { id: 5, user_id: 2, month: BillingMonth("2025-11".into()), units: 150.0, rate_per_unit: Money(580), fixed_charge: Money(4000), amount_due: Money(127000), due_date: BillingDate(date(2025, 11, 26).expect("static seed date must be valid")), status: BillStatus::Paid, consumption_history: vec![], payments: vec![Payment { id: 2, month: BillingMonth("2025-11".into()), amount: Money(127000), paid_at: paid_at_november, status: PaymentStatus::Completed }] },
        ];

        Self { users: Arc::new(Mutex::new(users)), bills: Arc::new(Mutex::new(bills)) }
    }
}

fn date(year: i32, month: u32, day: u32) -> Option<NaiveDate> {
    NaiveDate::from_ymd_opt(year, month, day)
}

fn make_timestamp(year: i32, month: u32, day: u32, hour: u32, minute: u32, second: u32) -> Option<chrono::DateTime<Utc>> {
    NaiveDate::from_ymd_opt(year, month, day).and_then(|date| date.and_hms_opt(hour, minute, second)).map(|naive| Utc.from_utc_datetime(&naive))
}
