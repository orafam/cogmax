use cogmax_domain::{
    candidate::MemoryCandidate,
    memory::{Memory, MemoryStatus},
    scope::MemoryScope,
};
use rusqlite::{params, Connection};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("invalid stored memory: {0}")]
    InvalidMemory(String),
}

pub struct SqliteStore {
    connection: Connection,
}

impl SqliteStore {
    pub fn open_path(path: impl AsRef<std::path::Path>) -> Result<Self, StorageError> {
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| StorageError::InvalidMemory(error.to_string()))?;
        }
        Self::open(Connection::open(path)?)
    }

    pub fn in_memory() -> Result<Self, StorageError> {
        Self::open(Connection::open_in_memory()?)
    }

    pub fn open(connection: Connection) -> Result<Self, StorageError> {
        let store = Self { connection };
        store.connection.execute_batch(
            "CREATE TABLE IF NOT EXISTS candidates (
                idempotency_key TEXT PRIMARY KEY,
                payload TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS memories (
                id TEXT PRIMARY KEY,
                scope TEXT NOT NULL,
                status TEXT NOT NULL,
                payload TEXT NOT NULL
            );",
        )?;
        Ok(store)
    }

    pub fn insert_candidate(&self, candidate: &MemoryCandidate) -> Result<bool, StorageError> {
        let changed = self.connection.execute(
            "INSERT OR IGNORE INTO candidates (idempotency_key, payload) VALUES (?1, ?2)",
            params![
                candidate.idempotency_key(),
                serde_json::to_string(candidate)?
            ],
        )?;
        Ok(changed == 1)
    }

    pub fn insert_memory(&self, memory: &Memory) -> Result<(), StorageError> {
        self.connection.execute(
            "INSERT OR REPLACE INTO memories (id, scope, status, payload) VALUES (?1, ?2, ?3, ?4)",
            params![
                memory.id.to_string(),
                memory.scope.as_str(),
                serde_json::to_string(&memory.status)?,
                serde_json::to_string(memory)?
            ],
        )?;
        Ok(())
    }

    pub fn list_active(&self, scope: &MemoryScope) -> Result<Vec<Memory>, StorageError> {
        self.list_active_where("=", scope.as_str())
    }

    pub fn list_all(&self) -> Result<Vec<Memory>, StorageError> {
        let mut statement = self
            .connection
            .prepare("SELECT payload FROM memories ORDER BY id")?;
        let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
        rows.map(|row| {
            let payload = row?;
            serde_json::from_str(&payload)
                .map_err(|error| StorageError::InvalidMemory(error.to_string()))
        })
        .collect()
    }

    pub fn list_active_prefix(&self, prefix: &str) -> Result<Vec<Memory>, StorageError> {
        self.list_active_where("LIKE", &format!("{prefix}%"))
    }

    fn list_active_where(&self, operator: &str, scope: &str) -> Result<Vec<Memory>, StorageError> {
        let mut statement = self.connection.prepare(&format!(
            "SELECT payload FROM memories WHERE scope {operator} ?1 AND status = ?2 ORDER BY id"
        ))?;
        let rows = statement.query_map(
            params![scope, serde_json::to_string(&MemoryStatus::Active)?],
            |row| row.get::<_, String>(0),
        )?;
        rows.map(|row| {
            let payload = row?;
            serde_json::from_str(&payload).map_err(|error| {
                rusqlite::Error::FromSqlConversionFailure(
                    payload.len(),
                    rusqlite::types::Type::Text,
                    Box::new(error),
                )
            })
        })
        .map(|memory| memory.map_err(StorageError::from))
        .collect()
    }

    pub fn set_status(&self, id: &uuid::Uuid, status: MemoryStatus) -> Result<(), StorageError> {
        let payload: String = self.connection.query_row(
            "SELECT payload FROM memories WHERE id = ?1",
            params![id.to_string()],
            |row| row.get(0),
        )?;
        let mut memory: Memory = serde_json::from_str(&payload)
            .map_err(|error| StorageError::InvalidMemory(error.to_string()))?;
        memory.status = status;
        self.connection.execute(
            "UPDATE memories SET status = ?1, payload = ?2 WHERE id = ?3",
            params![
                serde_json::to_string(&status)?,
                serde_json::to_string(&memory)?,
                id.to_string()
            ],
        )?;
        Ok(())
    }

    pub fn reset_inferred_imports(&self) -> Result<usize, StorageError> {
        let mut statement = self
            .connection
            .prepare("SELECT id, payload FROM memories WHERE status = ?1")?;
        let rows = statement.query_map(
            params![serde_json::to_string(&MemoryStatus::Active)?],
            |row| {
                let id: String = row.get(0)?;
                let payload: String = row.get(1)?;
                Ok((id, payload))
            },
        )?;
        let mut inferred_ids = Vec::new();
        for row in rows {
            let (id, payload) = row?;
            let memory: Memory = serde_json::from_str(&payload)
                .map_err(|error| StorageError::InvalidMemory(error.to_string()))?;
            if matches!(memory.authority, cogmax_domain::memory::Authority::Inferred) {
                inferred_ids.push(id);
            }
        }
        for id in &inferred_ids {
            self.connection
                .execute("DELETE FROM memories WHERE id = ?1", params![id])?;
        }
        self.connection.execute("DELETE FROM candidates", [])?;
        Ok(inferred_ids.len())
    }
}

#[cfg(test)]
mod tests;
