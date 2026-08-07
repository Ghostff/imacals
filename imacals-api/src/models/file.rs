use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// All valid upload purposes. Stored as kebab-case strings in the `files.type` column.
/// Add a new variant here whenever a new upload use-case is introduced — never use raw strings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum FileType {
    UserSignature,       // user-signature       — fileable: users
    UserInitials,        // user-initials        — fileable: users
    UserProofOfFunds,    // user-proof-of-funds  — fileable: users
    ProductImage,        // product-image        — fileable: products
    OrderAttachment,     // order-attachment     — fileable: orders (proof of payment, waybill)
}

impl FileType {
    pub fn as_str(&self) -> &'static str {
        match self {
            FileType::UserSignature     => "user-signature",
            FileType::UserInitials      => "user-initials",
            FileType::UserProofOfFunds  => "user-proof-of-funds",
            FileType::ProductImage      => "product-image",
            FileType::OrderAttachment   => "order-attachment",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    pub id:            Uuid,
    pub created_by:    Uuid,
    pub fileable_type: String,
    pub fileable_id:   Uuid,
    pub file_type:     FileType,
    pub name:          String,
    pub absolute_path: String,
    pub relative_path: String,
    pub size:          i64,
    pub mime_type:     String,
    pub created_at:    DateTime<Utc>,
    pub updated_at:    DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at:    Option<DateTime<Utc>>,
}

pub struct CreateFileInput {
    pub created_by:    Uuid,
    pub fileable_type: String,
    pub fileable_id:   Uuid,
    pub file_type:     FileType,
    pub name:          String,
    pub absolute_path: String,
    pub relative_path: String,
    pub size:          i64,
    pub mime_type:     String,
}
