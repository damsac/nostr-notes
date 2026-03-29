use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub pubkey: String,
    pub content: String,
    pub created_at: i64,
}
