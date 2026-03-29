use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};

use crate::error::Error;
use crate::models::Item;

pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn new(data_dir: &str) -> Result<Self, Error> {
        let db_path = format!("{}/app.db", data_dir);
        let mut conn = Connection::open(&db_path)?;

        let migrations = Migrations::new(vec![
            M::up("CREATE TABLE IF NOT EXISTS items (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );"),
        ]);
        migrations.to_latest(&mut conn)?;

        Ok(Self { conn })
    }

    pub fn create_item(&self, name: &str) -> Result<Item, Error> {
        let id = uuid::Uuid::now_v7().to_string();
        self.conn.execute(
            "INSERT INTO items (id, name) VALUES (?1, ?2)",
            rusqlite::params![id, name],
        )?;
        self.get_item(&id)
    }

    pub fn get_item(&self, id: &str) -> Result<Item, Error> {
        self.conn
            .query_row(
                "SELECT id, name, created_at FROM items WHERE id = ?1",
                rusqlite::params![id],
                |row| {
                    Ok(Item {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        created_at: row.get(2)?,
                    })
                },
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    Error::NotFound(format!("Item {}", id))
                }
                other => Error::Database(other),
            })
    }

    pub fn list_items(&self) -> Result<Vec<Item>, Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, created_at FROM items ORDER BY created_at DESC")?;
        let items = stmt
            .query_map([], |row| {
                Ok(Item {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    created_at: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_get_item() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::new(dir.path().to_str().unwrap()).unwrap();

        let item = store.create_item("Test Item").unwrap();
        assert_eq!(item.name, "Test Item");

        let fetched = store.get_item(&item.id).unwrap();
        assert_eq!(fetched.id, item.id);
        assert_eq!(fetched.name, "Test Item");
    }

    #[test]
    fn test_list_items() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::new(dir.path().to_str().unwrap()).unwrap();

        store.create_item("Item 1").unwrap();
        store.create_item("Item 2").unwrap();

        let items = store.list_items().unwrap();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::new(dir.path().to_str().unwrap()).unwrap();

        let result = store.get_item("nonexistent");
        assert!(matches!(result, Err(Error::NotFound(_))));
    }
}
