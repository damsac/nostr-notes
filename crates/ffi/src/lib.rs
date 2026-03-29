uniffi::setup_scaffolding!();

use nostr_notes_core::{relative_time, Error as CoreError, Note, RelayClient};
use std::sync::Mutex;
use tokio::runtime::Runtime;

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum FfiError {
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Could not connect to relays: {0}")]
    Relay(String),
    #[error("Something went wrong: {0}")]
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
    /// Resolved profile name or truncated npub (never empty).
    pub display_name: String,
    /// Human-readable relative timestamp (e.g. "2m ago", "3h ago").
    pub relative_time: String,
}

impl From<Note> for FfiNote {
    fn from(n: Note) -> Self {
        let rel = relative_time(n.created_at);
        FfiNote {
            id: n.id,
            pubkey: n.pubkey,
            content: n.content,
            created_at: n.created_at,
            display_name: n.display_name,
            relative_time: rel,
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
    /// Create a new AppCore connected to the given relay URL (plus built-in public relays).
    ///
    /// Pass an empty string to connect only to the default public relays.
    #[uniffi::constructor]
    pub fn new(relay_url: String, data_dir: String) -> Result<Self, FfiError> {
        let rt = Runtime::new().map_err(|e| FfiError::Internal(e.to_string()))?;
        let urls: Vec<&str> = if relay_url.is_empty() {
            vec![]
        } else {
            vec![relay_url.as_str()]
        };
        let client = rt
            .block_on(RelayClient::new(&urls, &data_dir))
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
