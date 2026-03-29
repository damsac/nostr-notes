uniffi::setup_scaffolding!();

use nostr_notes_core::{Error as CoreError, Item, Store};
use std::sync::Mutex;

/// FFI error type — maps from core errors to UniFFI-exportable errors.
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum FfiError {
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<CoreError> for FfiError {
    fn from(e: CoreError) -> Self {
        match e {
            CoreError::NotFound(msg) => FfiError::NotFound(msg),
            CoreError::InvalidInput(msg) => FfiError::InvalidInput(msg),
            other => FfiError::Internal(other.to_string()),
        }
    }
}

/// FFI record for Item — maps to/from core Item type.
#[derive(uniffi::Record)]
pub struct FfiItem {
    pub id: String,
    pub name: String,
    pub created_at: String,
}

impl From<Item> for FfiItem {
    fn from(item: Item) -> Self {
        FfiItem {
            id: item.id,
            name: item.name,
            created_at: item.created_at,
        }
    }
}

/// The main app object exposed to native code via UniFFI.
#[derive(uniffi::Object)]
pub struct AppCore {
    store: Mutex<Store>,
}

#[uniffi::export]
impl AppCore {
    #[uniffi::constructor]
    pub fn new(data_dir: String) -> Result<Self, FfiError> {
        let store = Store::new(&data_dir)?;
        Ok(Self {
            store: Mutex::new(store),
        })
    }

    pub fn create_item(&self, name: String) -> Result<FfiItem, FfiError> {
        let store = self.store.lock().unwrap();
        let item = store.create_item(&name)?;
        Ok(item.into())
    }

    pub fn get_item(&self, id: String) -> Result<FfiItem, FfiError> {
        let store = self.store.lock().unwrap();
        let item = store.get_item(&id)?;
        Ok(item.into())
    }

    pub fn list_items(&self) -> Result<Vec<FfiItem>, FfiError> {
        let store = self.store.lock().unwrap();
        let items = store.list_items()?;
        Ok(items.into_iter().map(|i| i.into()).collect())
    }
}
