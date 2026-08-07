use chrono::{DateTime, Utc};
use serde::{Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct OrganizationUser {
    pub id: Uuid,
    pub user_id: Uuid,
    pub organization_id: Uuid,
    pub added_by: Uuid,
    #[serde(skip_serializing)]
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing)]
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing)]
    pub deleted_at: Option<DateTime<Utc>>,
}
