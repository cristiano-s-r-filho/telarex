use rusqlite::{params, Connection};
use std::path::PathBuf;
use uuid::Uuid;
use anyhow::Result;
use crate::errors::TrexError;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open() -> Result<Self> {
        let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("telarex");
        std::fs::create_dir_all(&path)?;
        path.push("telarex.db");
        
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

    pub fn log_error(&self, error: &TrexError) -> Result<()> {
        let level_str = format!("{:?}", error.level);
        self.conn.execute(
            "INSERT INTO network_logs (code, level, message) VALUES (?1, ?2, ?3)",
            params![error.code, level_str, error.message],
        )?;
        Ok(())
    }

    pub fn register_lodge(&self, id: Uuid, path: &str, name: &str, is_owner: bool) -> Result<()> {
        // HARDENING: Deduplicate by UUID
        self.conn.execute(
            "INSERT OR REPLACE INTO lodges (uuid, path, name, is_owner, last_accessed)
             VALUES (?1, ?2, ?3, ?4, CURRENT_TIMESTAMP)",
            params![id.to_string(), path, name, if is_owner { 1 } else { 0 }],
        )?;
        Ok(())
    }

    pub fn add_recent_project(&self, path: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO recent_projects (path, last_opened) VALUES (?1, CURRENT_TIMESTAMP)",
            params![path],
        )?;
        Ok(())
    }

    pub fn get_recent_projects(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare("SELECT path FROM recent_projects ORDER BY last_opened DESC LIMIT 20")?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn register_session(&self, lodge_id: Uuid, peer_id: &str, username: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO sessions (lodge_id, peer_id, username) VALUES (?1, ?2, ?3)",
            params![lodge_id.to_string(), peer_id, username],
        )?;
        Ok(())
    }

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

    pub fn reset(&self) -> Result<()> {
        self.conn.execute("DROP TABLE IF EXISTS lodges", [])?;
        self.conn.execute("DROP TABLE IF EXISTS sessions", [])?;
        self.conn.execute("DROP TABLE IF EXISTS recent_projects", [])?;
        self.conn.execute("DROP TABLE IF EXISTS network_logs", [])?;
        self.conn.execute("DROP TABLE IF EXISTS access_control", [])?;
        self.initialize()?;
        Ok(())
    }

    pub fn delete_lodge(&self, id: Uuid) -> Result<()> {
        self.conn.execute("DELETE FROM lodges WHERE uuid = ?1", params![id.to_string()])?;
        Ok(())
    }
}
