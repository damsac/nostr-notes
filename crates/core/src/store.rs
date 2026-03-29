use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};

use crate::error::Error;
use crate::models::Note;

pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn new(data_dir: &str) -> Result<Self, Error> {
        let db_path = format!("{}/notes.db", data_dir);
        let mut conn = Connection::open(&db_path)?;

        let migrations = Migrations::new(vec![M::up(
            "CREATE TABLE IF NOT EXISTS notes (
                id TEXT PRIMARY KEY,
                pubkey TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_notes_created_at ON notes(created_at DESC);",
        )]);
        migrations.to_latest(&mut conn)?;

        Ok(Self { conn })
    }

    pub fn upsert_note(&self, note: &Note) -> Result<(), Error> {
        self.conn.execute(
            "INSERT OR IGNORE INTO notes (id, pubkey, content, created_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![note.id, note.pubkey, note.content, note.created_at],
        )?;
        Ok(())
    }

    pub fn list_notes(&self, limit: u32) -> Result<Vec<Note>, Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, pubkey, content, created_at FROM notes ORDER BY created_at DESC LIMIT ?1",
        )?;
        let notes = stmt
            .query_map(rusqlite::params![limit], |row| {
                Ok(Note {
                    id: row.get(0)?,
                    pubkey: row.get(1)?,
                    content: row.get(2)?,
                    created_at: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(notes)
    }

    pub fn note_count(&self) -> Result<u32, Error> {
        let count: u32 = self
            .conn
            .query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))?;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upsert_and_list() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::new(dir.path().to_str().unwrap()).unwrap();

        let note = Note {
            id: "abc123".into(),
            pubkey: "npub1test".into(),
            content: "Hello Nostr!".into(),
            created_at: 1700000000,
        };
        store.upsert_note(&note).unwrap();

        let notes = store.list_notes(50).unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].content, "Hello Nostr!");
    }

    #[test]
    fn test_idempotent_insert() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::new(dir.path().to_str().unwrap()).unwrap();

        let note = Note {
            id: "abc123".into(),
            pubkey: "npub1test".into(),
            content: "Hello Nostr!".into(),
            created_at: 1700000000,
        };
        store.upsert_note(&note).unwrap();
        store.upsert_note(&note).unwrap();

        assert_eq!(store.note_count().unwrap(), 1);
    }

    #[test]
    fn test_ordering() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::new(dir.path().to_str().unwrap()).unwrap();

        store
            .upsert_note(&Note {
                id: "older".into(),
                pubkey: "pk".into(),
                content: "old".into(),
                created_at: 1000,
            })
            .unwrap();
        store
            .upsert_note(&Note {
                id: "newer".into(),
                pubkey: "pk".into(),
                content: "new".into(),
                created_at: 2000,
            })
            .unwrap();

        let notes = store.list_notes(50).unwrap();
        assert_eq!(notes[0].id, "newer");
        assert_eq!(notes[1].id, "older");
    }
}
