uniffi::setup_scaffolding!();

use nostr_notes_core::{Error as CoreError, Note, RelayClient};
use std::sync::Mutex;
use tokio::runtime::Runtime;

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum FfiError {
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Relay error: {0}")]
    Relay(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<CoreError> for FfiError {
    fn from(e: CoreError) -> Self {
        match e {
            CoreError::NotFound(msg) => FfiError::NotFound(msg),
            CoreError::Relay(msg) => FfiError::Relay(msg),
            other => FfiError::Internal(other.to_string()),
        }
    }
}

#[derive(uniffi::Record)]
pub struct FfiNote {
    pub id: String,
    pub pubkey: String,
    pub content: String,
    pub created_at: i64,
}

impl From<Note> for FfiNote {
    fn from(n: Note) -> Self {
        FfiNote {
            id: n.id,
            pubkey: n.pubkey,
            content: n.content,
            created_at: n.created_at,
        }
    }
}

#[derive(uniffi::Object)]
pub struct AppCore {
    client: Mutex<RelayClient>,
    rt: Runtime,
}

#[uniffi::export]
impl AppCore {
    #[uniffi::constructor]
    pub fn new(relay_url: String, data_dir: String) -> Result<Self, FfiError> {
        let rt = Runtime::new().map_err(|e| FfiError::Internal(e.to_string()))?;
        let client = rt
            .block_on(RelayClient::new(&relay_url, &data_dir))
            .map_err(FfiError::from)?;
        Ok(Self {
            client: Mutex::new(client),
            rt,
        })
    }

    pub fn fetch_global_notes(&self, limit: u16) -> Result<Vec<FfiNote>, FfiError> {
        let client = self.client.lock().unwrap();
        let notes = self
            .rt
            .block_on(client.fetch_global_notes(limit))
            .map_err(FfiError::from)?;
        Ok(notes.into_iter().map(FfiNote::from).collect())
    }

    pub fn fetch_notes_by_pubkey(
        &self,
        pubkey_hex: String,
        limit: u16,
    ) -> Result<Vec<FfiNote>, FfiError> {
        let client = self.client.lock().unwrap();
        let notes = self
            .rt
            .block_on(client.fetch_notes_by_pubkey(&pubkey_hex, limit))
            .map_err(FfiError::from)?;
        Ok(notes.into_iter().map(FfiNote::from).collect())
    }

    pub fn cached_notes(&self, limit: u32) -> Result<Vec<FfiNote>, FfiError> {
        let client = self.client.lock().unwrap();
        let notes = client.cached_notes(limit).map_err(FfiError::from)?;
        Ok(notes.into_iter().map(FfiNote::from).collect())
    }
}
