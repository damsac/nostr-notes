use nostr_sdk::prelude::*;
use std::collections::HashMap;
use std::time::Duration;

use crate::error::Error;
use crate::models::{truncated_npub, Note};
use crate::store::Store;

pub struct RelayClient {
    client: Client,
    store: Store,
}

fn event_to_note(event: &Event, display_name: &str) -> Note {
    Note {
        id: event.id.to_hex(),
        pubkey: event.pubkey.to_hex(),
        content: event.content.to_string(),
        created_at: event.created_at.as_u64() as i64,
        display_name: display_name.to_string(),
    }
}

/// Default public relays that are known to have content.
const DEFAULT_RELAYS: &[&str] = &[
    "wss://relay.damus.io",
    "wss://nos.lol",
    "wss://relay.nostr.band",
];

impl RelayClient {
    /// Create a new relay client connected to the given relays plus defaults.
    ///
    /// `relay_urls` may be empty — the default public relays are always included.
    pub async fn new(relay_urls: &[&str], data_dir: &str) -> Result<Self, Error> {
        let store = Store::new(data_dir)?;
        let client = Client::default();

        for url in relay_urls.iter().chain(DEFAULT_RELAYS.iter()) {
            if let Err(e) = client.add_relay(*url).await {
                log::warn!("failed to add relay {}: {}", url, e);
            }
        }

        client.connect().await;
        Ok(Self { client, store })
    }

    /// Resolve display names for a set of pubkeys by fetching kind 0 metadata.
    /// Returns a map of hex pubkey -> display name. Falls back to truncated npub
    /// for pubkeys whose metadata could not be resolved.
    async fn resolve_display_names(&self, pubkeys: &[PublicKey]) -> HashMap<String, String> {
        let mut names: HashMap<String, String> = HashMap::new();
        if pubkeys.is_empty() {
            return names;
        }

        // Batch-fetch kind 0 metadata for all unique pubkeys (short timeout).
        let filter = Filter::new()
            .authors(pubkeys.iter().copied())
            .kind(Kind::Metadata)
            .limit(pubkeys.len());

        if let Ok(events) = self
            .client
            .fetch_events(filter, Duration::from_secs(5))
            .await
        {
            // Keep only the newest metadata per pubkey.
            let mut best: HashMap<String, &Event> = HashMap::new();
            for event in events.iter() {
                let hex = event.pubkey.to_hex();
                let is_newer = best
                    .get(&hex)
                    .is_none_or(|prev| event.created_at > prev.created_at);
                if is_newer {
                    best.insert(hex, event);
                }
            }
            for (hex, event) in &best {
                if let Ok(meta) = Metadata::try_from(*event) {
                    let name = meta
                        .display_name
                        .or(meta.name)
                        .unwrap_or_else(|| truncated_npub(hex));
                    names.insert(hex.clone(), name);
                }
            }
        }

        // Fill in any missing pubkeys with truncated npub.
        for pk in pubkeys {
            let hex = pk.to_hex();
            names
                .entry(hex.clone())
                .or_insert_with(|| truncated_npub(&hex));
        }

        names
    }

    pub async fn fetch_global_notes(&self, limit: u16) -> Result<Vec<Note>, Error> {
        let filter = Filter::new().kind(Kind::TextNote).limit(limit as usize);

        let events = self
            .client
            .fetch_events(filter, Duration::from_secs(10))
            .await
            .map_err(|e| Error::Relay(e.to_string()))?;

        // Collect unique pubkeys and resolve display names.
        let pubkeys: Vec<PublicKey> = events
            .iter()
            .map(|e| e.pubkey)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        let names = self.resolve_display_names(&pubkeys).await;

        let mut notes = Vec::new();
        for event in events.iter() {
            let hex = event.pubkey.to_hex();
            let display = names
                .get(&hex)
                .cloned()
                .unwrap_or_else(|| truncated_npub(&hex));
            let note = event_to_note(event, &display);
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
        let pubkey = PublicKey::from_hex(pubkey_hex).map_err(|e| Error::Relay(e.to_string()))?;

        let filter = Filter::new()
            .author(pubkey)
            .kind(Kind::TextNote)
            .limit(limit as usize);

        let events = self
            .client
            .fetch_events(filter, Duration::from_secs(10))
            .await
            .map_err(|e| Error::Relay(e.to_string()))?;

        let names = self.resolve_display_names(&[pubkey]).await;
        let display = names
            .get(pubkey_hex)
            .cloned()
            .unwrap_or_else(|| truncated_npub(pubkey_hex));

        let mut notes = Vec::new();
        for event in events.iter() {
            let note = event_to_note(event, &display);
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
