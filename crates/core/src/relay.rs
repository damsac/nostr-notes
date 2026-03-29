use nostr_sdk::prelude::*;
use std::time::Duration;

use crate::error::Error;
use crate::models::Note;
use crate::store::Store;

pub struct RelayClient {
    client: Client,
    store: Store,
}

fn event_to_note(event: &Event) -> Note {
    Note {
        id: event.id.to_hex(),
        pubkey: event.pubkey.to_hex(),
        content: event.content.to_string(),
        created_at: event.created_at.as_u64() as i64,
    }
}

impl RelayClient {
    pub async fn new(relay_url: &str, data_dir: &str) -> Result<Self, Error> {
        let store = Store::new(data_dir)?;
        let client = Client::default();
        client
            .add_relay(relay_url)
            .await
            .map_err(|e| Error::Relay(e.to_string()))?;
        client.connect().await;
        Ok(Self { client, store })
    }

    pub async fn fetch_global_notes(&self, limit: u16) -> Result<Vec<Note>, Error> {
        let filter = Filter::new().kind(Kind::TextNote).limit(limit as usize);

        let events = self
            .client
            .fetch_events(filter, Duration::from_secs(10))
            .await
            .map_err(|e| Error::Relay(e.to_string()))?;

        let mut notes = Vec::new();
        for event in events.iter() {
            let note = event_to_note(event);
            self.store.upsert_note(&note)?;
            notes.push(note);
        }

        Ok(notes)
    }

    pub async fn fetch_notes_by_pubkey(
        &self,
        pubkey_hex: &str,
        limit: u16,
    ) -> Result<Vec<Note>, Error> {
        let pubkey =
            PublicKey::from_hex(pubkey_hex).map_err(|e| Error::Relay(e.to_string()))?;

        let filter = Filter::new()
            .author(pubkey)
            .kind(Kind::TextNote)
            .limit(limit as usize);

        let events = self
            .client
            .fetch_events(filter, Duration::from_secs(10))
            .await
            .map_err(|e| Error::Relay(e.to_string()))?;

        let mut notes = Vec::new();
        for event in events.iter() {
            let note = event_to_note(event);
            self.store.upsert_note(&note)?;
            notes.push(note);
        }

        Ok(notes)
    }

    pub fn cached_notes(&self, limit: u32) -> Result<Vec<Note>, Error> {
        self.store.list_notes(limit)
    }

    pub async fn disconnect(&self) {
        self.client.disconnect().await;
    }
}
