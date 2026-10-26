use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: u32,
    pub name: String,
    pub email: String,
    pub meter_number: String,
    pub address: String,
    pub role: Role,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Admin,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Money(i64);

impl Money {
    pub fn from_cents(cents: i64) -> Result<Self, String> {
        if cents < 0 {
            Err("money must be non-negative".into())
        } else {
            Ok(Self(cents))
        }
    }

    pub fn from_rupees_str(value: &str) -> Result<Self, String> {
        let trimmed = value.trim();
        if trimmed.starts_with('-') {
            return Err("money must be non-negative".into());
        }
        let mut parts = trimmed.split('.');
        let whole = parts.next().ok_or_else(|| "money must be numeric".to_string())?;
        let frac = parts.next().unwrap_or("");
        if parts.next().is_some() || whole.is_empty() || !whole.chars().all(|c| c.is_ascii_digit()) {
            return Err("money must be a decimal amount with at most two fractional digits".into());
        }
        if !frac.chars().all(|c| c.is_ascii_digit()) || frac.len() > 2 {
            return Err("money must be a decimal amount with at most two fractional digits".into());
        }
        let whole = whole.parse::<i64>().map_err(|_| "money is too large".to_string())?;
        let cents = whole
            .checked_mul(100)
            .ok_or_else(|| "money is too large".to_string())?
            .checked_add(match frac.len() {
                0 => 0,
                1 => frac.parse::<i64>().map_err(|_| "money is too large".to_string())? * 10,
                _ => frac.parse::<i64>().map_err(|_| "money is too large".to_string())?,
            })
            .ok_or_else(|| "money is too large".to_string())?;
        Self::from_cents(cents)
    }

    pub fn as_cents(&self) -> i64 {
        self.0
    }

    pub fn as_rupees(&self) -> String {
        debug_assert!(self.0 >= 0, "money invariant violated");
        format!("{}.{:02}", self.0 / 100, self.0 % 100)
    }
}

impl TryFrom<i64> for Money {
    type Error = String;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Self::from_cents(value)
    }
}

impl Serialize for Money {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.as_rupees())
    }
}

impl<'de> Deserialize<'de> for Money {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Money::from_rupees_str(&value).map_err(serde::de::Error::custom)
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.as_rupees())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BillingMonth {
    pub year: i32,
    pub month: u32,
}

impl BillingMonth {
    pub fn parse(value: impl AsRef<str>) -> Result<Self, String> {
        let value = value.as_ref();
        let mut parts = value.split('-');
        let year = parts.next().ok_or_else(|| "billing month must be in YYYY-MM format".to_string())?;
        let month = parts.next().ok_or_else(|| "billing month must be in YYYY-MM format".to_string())?;
        if parts.next().is_some() || year.len() != 4 || month.len() != 2 {
            return Err("billing month must be in YYYY-MM format".into());
        }
        if !year.chars().all(|c| c.is_ascii_digit()) || !month.chars().all(|c| c.is_ascii_digit()) {
            return Err("billing month must contain digits only".into());
        }
        let year = year.parse::<i32>().map_err(|_| "billing month year is invalid".to_string())?;
        let month = month.parse::<u32>().map_err(|_| "billing month month is invalid".to_string())?;
        if !(2000..=2099).contains(&year) {
            return Err("billing month year must be between 2000 and 2099".into());
        }
        if !(1..=12).contains(&month) {
            return Err("billing month must be between 01 and 12".into());
        }
        Ok(Self { year, month })
    }
}

impl fmt::Display for BillingMonth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}", self.year, self.month)
    }
}

impl Serialize for BillingMonth {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for BillingMonth {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        BillingMonth::parse(&value).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BillingDate(pub NaiveDate);

impl BillingDate {
    pub fn parse(value: impl AsRef<str>) -> Result<Self, String> {
        NaiveDate::parse_from_str(value.as_ref(), "%Y-%m-%d")
            .map(Self)
            .map_err(|err| format!("invalid billing date: {err}"))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ConsumptionRecord {
    pub month: BillingMonth,
    pub units: u32,
    pub rate_per_unit: Money,
    pub fixed_charge: Money,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum PaymentStatus {
    Completed,
    Failed,
    Refunded,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Payment {
    pub id: u32,
    pub month: BillingMonth,
    pub amount: Money,
    pub paid_at: DateTime<Utc>,
    pub status: PaymentStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Bill {
    pub id: u32,
    pub user_id: u32,
    pub month: BillingMonth,
    pub units: u32,
    pub rate_per_unit: Money,
    pub fixed_charge: Money,
    pub amount_due: Money,
    pub due_date: BillingDate,
    pub status: BillStatus,
    pub consumption_history: Vec<ConsumptionRecord>,
    pub payments: Vec<Payment>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BillStatus {
    Pending,
    Paid,
    Overdue,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewBillInput {
    pub user_id: u32,
    pub month: String,
    pub units: u32,
    pub rate_per_unit: String,
    pub fixed_charge: String,
    pub due_date: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewPaymentInput {
    pub amount: String,
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
    pub total_units: u32,
    pub total_revenue: Money,
    pub average_bill: Money,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DashboardView {
    pub summary: StatsSummary,
    pub users: Vec<User>,
    pub bills: Vec<Bill>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub message: String,
}
