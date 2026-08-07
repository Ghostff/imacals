use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Describes WHAT a user IS within an org — contractor, broker, realtor, etc.
// Distinct from roles (admin, super-admin) which control WHAT a user CAN DO.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationUserRole {
    pub id: Uuid,
    pub name: String,
    pub title: String,
    pub description: String,
    pub organization_id: Option<Uuid>,
    // true for the roles that may be assigned as system users (hml, insurance, broker, realtor).
    pub system_user_eligible: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<DateTime<Utc>>,
}

// Lightweight summary embedded in user responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationUserRoleSummary {
    pub id: Uuid,
    pub name: String,
    pub title: String,
}
