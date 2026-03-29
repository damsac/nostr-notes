use serde::{Deserialize, Serialize};

/// Example model — replace with your domain types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub created_at: String,
}
