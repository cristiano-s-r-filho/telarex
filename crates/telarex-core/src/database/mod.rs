//! Database — SQLite persistence for lodges, sessions, recent projects, and logs.
//!
//! [`Database`] manages the local SQLite database which stores lodge registrations,
//! peer sessions, recent project history, network error logs, and access control
//! entries. It is used by the networking and workspace layers for durable state.

use rusqlite::{params, Connection};
use uuid::Uuid;
use anyhow::Result;
use crate::errors::TrexError;

/// SQLite database wrapper for local persistence of lodges, sessions, and logs.
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open (or create) the database, running schema initialization.
    pub fn open() -> Result<Self> {
        let data_dir = directories::ProjectDirs::from("", "", "telarex")
            .ok_or_else(|| anyhow::anyhow!("could not determine data directory"))?
            .data_dir()
            .to_path_buf();
        std::fs::create_dir_all(&data_dir)?;
        let path = data_dir.join("telarex.db");
        
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.initialize()?;
        Ok(db)
    }

    fn initialize(&self) -> Result<()> {
        self.conn.execute_batch(
            "PRAGMA foreign_keys = ON;
            BEGIN;
            CREATE TABLE IF NOT EXISTS lodges (
                uuid TEXT PRIMARY KEY,
                path TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                is_owner INTEGER NOT NULL,
                last_accessed DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                peer_id TEXT NOT NULL,
                username TEXT NOT NULL,
                lodge_id TEXT NOT NULL,
                joined_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(lodge_id, peer_id),
                FOREIGN KEY(lodge_id) REFERENCES lodges(uuid) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS recent_projects (
                path TEXT PRIMARY KEY,
                last_opened DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            CREATE TABLE IF NOT EXISTS network_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                code TEXT NOT NULL,
                level TEXT NOT NULL,
                message TEXT NOT NULL,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            CREATE TABLE IF NOT EXISTS access_control (
                lodge_id TEXT NOT NULL,
                peer_id TEXT NOT NULL,
                authorized INTEGER NOT NULL DEFAULT 1,
                PRIMARY KEY(lodge_id, peer_id),
                FOREIGN KEY(lodge_id) REFERENCES lodges(uuid) ON DELETE CASCADE
            );
            COMMIT;"
        )?;
        Ok(())
    }

    /// Persist a [`TrexError`] to the network_logs table.
    pub fn log_error(&self, error: &TrexError) -> Result<()> {
        let level_str = format!("{:?}", error.level);
        self.conn.execute(
            "INSERT INTO network_logs (code, level, message) VALUES (?1, ?2, ?3)",
            params![error.code, level_str, error.message],
        )?;
        Ok(())
    }

    /// Register or update a lodge record in the database.
    pub fn register_lodge(&self, id: Uuid, path: &str, name: &str, is_owner: bool) -> Result<()> {
        // HARDENING: Deduplicate by UUID
        self.conn.execute(
            "INSERT OR REPLACE INTO lodges (uuid, path, name, is_owner, last_accessed)
             VALUES (?1, ?2, ?3, ?4, CURRENT_TIMESTAMP)",
            params![id.to_string(), path, name, if is_owner { 1 } else { 0 }],
        )?;
        Ok(())
    }

    /// Add a path to the recent projects list.
    pub fn add_recent_project(&self, path: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO recent_projects (path, last_opened) VALUES (?1, CURRENT_TIMESTAMP)",
            params![path],
        )?;
        Ok(())
    }

    /// Return the 20 most recent project paths, ordered by last opened.
    pub fn get_recent_projects(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare("SELECT path FROM recent_projects ORDER BY last_opened DESC LIMIT 20")?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Register or update a peer session for a lodge.
    pub fn register_session(&self, lodge_id: Uuid, peer_id: &str, username: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO sessions (lodge_id, peer_id, username) VALUES (?1, ?2, ?3)",
            params![lodge_id.to_string(), peer_id, username],
        )?;
        Ok(())
    }

    /// Return all lodges owned by the local user.
    pub fn get_my_lodges(&self) -> Result<Vec<(Uuid, String, String)>> {
        let mut stmt = self.conn.prepare("SELECT uuid, path, name FROM lodges WHERE is_owner = 1 ORDER BY last_accessed DESC")?;
        let rows = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            Ok((
                Uuid::parse_str(&id_str).unwrap_or_default(),
                row.get(1)?,
                row.get(2)?,
            ))
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Drop all tables and re‑initialize the database schema.
    pub fn reset(&self) -> Result<()> {
        self.conn.execute("DROP TABLE IF EXISTS lodges", [])?;
        self.conn.execute("DROP TABLE IF EXISTS sessions", [])?;
        self.conn.execute("DROP TABLE IF EXISTS recent_projects", [])?;
        self.conn.execute("DROP TABLE IF EXISTS network_logs", [])?;
        self.conn.execute("DROP TABLE IF EXISTS access_control", [])?;
        self.initialize()?;
        Ok(())
    }

    /// Delete a lodge and its cascade‑related sessions/access control entries.
    pub fn delete_lodge(&self, id: Uuid) -> Result<()> {
        self.conn.execute("DELETE FROM lodges WHERE uuid = ?1", params![id.to_string()])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn test_db() -> Database {
        let conn = Connection::open_in_memory().unwrap();
        let db = Database { conn };
        db.initialize().unwrap();
        db
    }

    #[test]
    fn test_initialize_creates_tables() {
        let db = test_db();
        let tables: Vec<String> = db
            .conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert!(tables.contains(&"access_control".to_string()));
        assert!(tables.contains(&"lodges".to_string()));
        assert!(tables.contains(&"network_logs".to_string()));
        assert!(tables.contains(&"recent_projects".to_string()));
        assert!(tables.contains(&"sessions".to_string()));
    }

    #[test]
    fn test_register_and_retrieve_lodges() {
        let db = test_db();
        let id = Uuid::new_v4();
        db.register_lodge(id, "/tmp/test", "Test Lodge", true).unwrap();

        let lodges = db.get_my_lodges().unwrap();
        assert_eq!(lodges.len(), 1);
        assert_eq!(lodges[0].0, id);
        assert_eq!(lodges[0].1, "/tmp/test");
        assert_eq!(lodges[0].2, "Test Lodge");
    }

    #[test]
    fn test_register_lodge_deduplicate() {
        let db = test_db();
        let id = Uuid::new_v4();
        db.register_lodge(id, "/dup", "First", true).unwrap();
        db.register_lodge(id, "/dup", "Second", true).unwrap();

        let lodges = db.get_my_lodges().unwrap();
        assert_eq!(lodges.len(), 1);
        assert_eq!(lodges[0].2, "Second");
    }

    #[test]
    fn test_delete_lodge_cascades() {
        let db = test_db();
        let id = Uuid::new_v4();
        db.register_lodge(id, "/cascade", "Cascade Lodge", true).unwrap();
        db.register_session(id, "peer1", "Alice").unwrap();

        db.delete_lodge(id).unwrap();

        let lodges = db.get_my_lodges().unwrap();
        assert!(lodges.is_empty());
        let _ = db.register_lodge(id, "/new", "New Lodge", true).unwrap();
    }

    #[test]
    fn test_recent_projects_crud() {
        let db = test_db();

        let projects = db.get_recent_projects().unwrap();
        assert!(projects.is_empty());

        db.add_recent_project("/path/one").unwrap();
        db.add_recent_project("/path/two").unwrap();
        db.add_recent_project("/path/one").unwrap();

        let projects = db.get_recent_projects().unwrap();
        assert_eq!(projects.len(), 2);
    }

    #[test]
    fn test_session_registration() {
        let db = test_db();
        let id = Uuid::new_v4();
        db.register_lodge(id, "/session", "Session Lodge", true).unwrap();

        db.register_session(id, "alice@device1", "Alice").unwrap();
        db.register_session(id, "bob@device2", "Bob").unwrap();

        let lodges = db.get_my_lodges().unwrap();
        assert_eq!(lodges.len(), 1);
    }

    #[test]
    fn test_reset_clears_everything() {
        let db = test_db();
        let id = Uuid::new_v4();
        db.register_lodge(id, "/reset", "Reset Lodge", true).unwrap();
        db.add_recent_project("/reset/path").unwrap();

        db.reset().unwrap();

        assert!(db.get_my_lodges().unwrap().is_empty());
        assert!(db.get_recent_projects().unwrap().is_empty());
    }

    #[test]
    fn test_log_error() {
        let db = test_db();
        let err = TrexError::network_failure("test");
        db.log_error(&err).unwrap();

        let count: i64 = db
            .conn
            .query_row("SELECT COUNT(*) FROM network_logs", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }
}
