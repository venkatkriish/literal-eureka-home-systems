use std::sync::{Arc, Mutex};

use crate::models::{Bill, BillStatus, ConsumptionRecord, Payment, Role, User};

#[derive(Clone)]
pub struct AppState {
    pub users: Arc<Mutex<Vec<User>>>,
    pub bills: Arc<Mutex<Vec<Bill>>>,
}

impl AppState {
    pub fn seeded() -> Self {
        let users = vec![
            User {
                id: 1,
                name: "Asha Kumar".into(),
                email: "asha@example.com".into(),
                meter_number: "MTR-1001".into(),
                address: "12 Lake Road".into(),
                role: Role::User,
            },
            User {
                id: 2,
                name: "Rahul Singh".into(),
                email: "rahul@example.com".into(),
                meter_number: "MTR-1002".into(),
                address: "44 Green Avenue".into(),
                role: Role::User,
            },
            User {
                id: 3,
                name: "Board Admin".into(),
                email: "admin@board.local".into(),
                meter_number: "ADMIN-01".into(),
                address: "Operations Center".into(),
                role: Role::Admin,
            },
        ];
        let bills = vec![
            Bill {
                id: 1,
                user_id: 1,
                month: "2026-01".into(),
                units: 210.0,
                rate_per_unit: 6.5,
                fixed_charge: 45.0,
                amount_due: 1410.0,
                due_date: "2026-01-28".into(),
                status: BillStatus::Pending,
                consumption_history: vec![
                    ConsumptionRecord { month: "2025-11".into(), units: 180.0, rate_per_unit: 6.0, fixed_charge: 45.0 },
                    ConsumptionRecord { month: "2025-12".into(), units: 195.0, rate_per_unit: 6.25, fixed_charge: 45.0 },
                ],
                payments: vec![],
            },
            Bill {
                id: 2,
                user_id: 1,
                month: "2025-12".into(),
                units: 195.0,
                rate_per_unit: 6.25,
                fixed_charge: 45.0,
                amount_due: 1263.75,
                due_date: "2025-12-28".into(),
                status: BillStatus::Paid,
                consumption_history: vec![],
                payments: vec![Payment { id: 1, month: "2025-12".into(), amount: 1263.75, paid_at: chrono::Utc::now(), status: "completed".into() }],
            },
            Bill {
                id: 3,
                user_id: 2,
                month: "2026-01".into(),
                units: 160.0,
                rate_per_unit: 5.9,
                fixed_charge: 40.0,
                amount_due: 988.0,
                due_date: "2026-01-25".into(),
                status: BillStatus::Pending,
                consumption_history: vec![],
                payments: vec![],
            },
        ];
        Self { users: Arc::new(Mutex::new(users)), bills: Arc::new(Mutex::new(bills)) }
    }
}
