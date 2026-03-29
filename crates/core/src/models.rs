use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub pubkey: String,
    pub content: String,
    pub created_at: i64,
    /// Human-friendly name: resolved from kind 0 metadata, or truncated npub.
    pub display_name: String,
}

/// Build a short display name from a hex pubkey by encoding it as npub and
/// truncating to `npub1xxxx...xxxx` (first 10 + last 4 chars).
pub fn truncated_npub(hex: &str) -> String {
    use nostr_sdk::prelude::*;
    match PublicKey::from_hex(hex) {
        Ok(pk) => match pk.to_bech32() {
            Ok(npub) => {
                if npub.len() > 16 {
                    format!("{}...{}", &npub[..10], &npub[npub.len() - 4..])
                } else {
                    npub
                }
            }
            Err(_) => format!("{}...", &hex[..hex.len().min(12)]),
        },
        Err(_) => format!("{}...", &hex[..hex.len().min(12)]),
    }
}

/// Compute a human-readable relative timestamp string (e.g. "2m ago", "3h ago").
pub fn relative_time(unix_secs: i64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let delta = now - unix_secs;
    if delta < 0 {
        return "just now".to_string();
    }
    let minutes = delta / 60;
    let hours = minutes / 60;
    let days = hours / 24;
    match () {
        _ if minutes < 1 => "just now".to_string(),
        _ if minutes < 60 => format!("{minutes}m ago"),
        _ if hours < 24 => format!("{hours}h ago"),
        _ if days < 30 => format!("{days}d ago"),
        _ => {
            let months = days / 30;
            if months < 12 {
                format!("{months}mo ago")
            } else {
                format!("{}y ago", days / 365)
            }
        }
    }
}
