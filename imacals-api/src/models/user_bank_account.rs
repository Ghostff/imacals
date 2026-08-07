use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBankAccount {
    pub id: Uuid,
    pub user_id: Uuid,
    pub bank_name: String,
    pub account_holder_name: String,
    pub account_type: String,
    pub account_number: String,
    pub routing_number: String,
    pub is_primary: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserBankAccountSchema {
    #[validate(length(min = 1, max = 200, message = "Bank name is required"))]
    pub bank_name: String,
    #[validate(length(min = 1, max = 200, message = "Account holder name is required"))]
    pub account_holder_name: String,
    /// checking or savings
    pub account_type: Option<String>,
    #[validate(length(min = 1, message = "Account number is required"))]
    pub account_number: String,
    #[validate(length(min = 1, message = "Routing number is required"))]
    pub routing_number: String,
    #[serde(default)]
    pub is_primary: bool,
}
