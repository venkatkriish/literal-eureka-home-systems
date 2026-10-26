use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{fmt, str::FromStr};
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Role {
    User,
    Admin,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub id: u32,
    pub name: String,
    pub email: String,
    pub meter_number: String,
    pub address: String,
    pub role: Role,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Money(pub i64);

impl Money {
    pub fn new(cents: i64) -> Result<Self, DomainError> {
        if cents < 0 {
            return Err(DomainError::NegativeMoney);
        }
        Ok(Self(cents))
    }

    pub fn zero() -> Self {
        Self(0)
    }

    pub fn cents(self) -> i64 {
        self.0
    }
}

impl Serialize for Money {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{:.2}", self.0 as f64 / 100.0))
    }
}

impl<'de> Deserialize<'de> for Money {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        parse_money(&raw).map_err(serde::de::Error::custom)
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}", self.0 as f64 / 100.0)
    }
}

pub fn parse_money(input: &str) -> Result<Money, DomainError> {
    let trimmed = input.trim().trim_start_matches('₹');
    let value: f64 = trimmed.parse().map_err(|_| DomainError::InvalidMoney)?;
    if value < 0.0 {
        return Err(DomainError::NegativeMoney);
    }
    Ok(Money((value * 100.0).round() as i64))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BillingMonth(pub String);

impl BillingMonth {
    pub fn parse(input: &str) -> Result<Self, DomainError> {
        let parts: Vec<_> = input.split('-').collect();
        if parts.len() != 2 {
            return Err(DomainError::InvalidMonth);
        }
        let year = parts[0].parse::<u32>().map_err(|_| DomainError::InvalidMonth)?;
        let month = parts[1].parse::<u32>().map_err(|_| DomainError::InvalidMonth)?;
        if !(2000..=2099).contains(&year) || !(1..=12).contains(&month) {
            return Err(DomainError::InvalidMonth);
        }
        Ok(Self(format!("{year:04}-{month:02}")))
    }
}

impl FromStr for BillingMonth {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl Serialize for BillingMonth {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BillingDate(pub NaiveDate);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BillStatus {
    Pending,
    Paid,
    Overdue,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PaymentStatus {
    Completed,
    Failed,
    Refunded,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConsumptionRecord {
    pub month: BillingMonth,
    pub units: f64,
    pub rate_per_unit: Money,
    pub fixed_charge: Money,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Payment {
    pub id: u32,
    pub month: BillingMonth,
    pub amount: Money,
    pub paid_at: DateTime<Utc>,
    pub status: PaymentStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bill {
    pub id: u32,
    pub user_id: u32,
    pub month: BillingMonth,
    pub units: f64,
    pub rate_per_unit: Money,
    pub fixed_charge: Money,
    pub amount_due: Money,
    pub due_date: BillingDate,
    pub status: BillStatus,
    pub consumption_history: Vec<ConsumptionRecord>,
    pub payments: Vec<Payment>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewBillInput {
    pub user_id: u32,
    pub month: String,
    pub units: f64,
    pub rate_per_unit: String,
    pub fixed_charge: String,
    pub due_date: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewPaymentInput {
    pub amount: String,
    pub status: Option<PaymentStatus>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateUserInput {
    pub name: Option<String>,
    pub email: Option<String>,
    pub meter_number: Option<String>,
    pub address: Option<String>,
    pub role: Option<Role>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StatsSummary {
    pub total_users: usize,
    pub total_bills: usize,
    pub paid_bills: usize,
    pub pending_bills: usize,
    pub overdue_bills: usize,
    pub total_units: f64,
    pub total_revenue: Money,
    pub average_bill: Money,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DashboardView {
    pub summary: StatsSummary,
    pub users: Vec<User>,
    pub bills: Vec<Bill>,
}

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("money must be non-negative")]
    NegativeMoney,
    #[error("invalid money value")]
    InvalidMoney,
    #[error("invalid billing month")]
    InvalidMonth,
    #[error("not found")]
    NotFound,
    #[error("validation failed: {0}")]
    Validation(String),
}

#[derive(Clone, Debug, Serialize)]
pub struct ApiError {
    pub message: String,
}
