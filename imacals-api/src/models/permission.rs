use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Permission {
    pub id: Uuid,
    pub name: String,
}
