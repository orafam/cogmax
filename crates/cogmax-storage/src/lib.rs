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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Supersession {
    pub replaced_id: uuid::Uuid,
    pub replacement_id: uuid::Uuid,
    pub reason: String,
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
            );
            CREATE TABLE IF NOT EXISTS supersessions (
                replaced_id TEXT PRIMARY KEY,
                replacement_id TEXT NOT NULL,
                reason TEXT NOT NULL
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

    pub fn supersede_memory(
        &self,
        replaced_id: &uuid::Uuid,
        replacement_id: &uuid::Uuid,
        reason: &str,
    ) -> Result<(), StorageError> {
        let active = serde_json::to_string(&MemoryStatus::Active)?;
        let replaced_status: String = self.connection.query_row(
            "SELECT status FROM memories WHERE id = ?1",
            params![replaced_id.to_string()],
            |row| row.get(0),
        )?;
        let replacement_status: String = self.connection.query_row(
            "SELECT status FROM memories WHERE id = ?1",
            params![replacement_id.to_string()],
            |row| row.get(0),
        )?;
        if replaced_status != active
            || replacement_status != active
            || replaced_id == replacement_id
        {
            return Err(StorageError::InvalidMemory(
                "supersession requires distinct active memories".into(),
            ));
        }
        self.set_status(replaced_id, MemoryStatus::Superseded)?;
        self.connection.execute(
            "INSERT OR REPLACE INTO supersessions (replaced_id, replacement_id, reason) VALUES (?1, ?2, ?3)",
            params![replaced_id.to_string(), replacement_id.to_string(), reason],
        )?;
        Ok(())
    }

    pub fn supersession(
        &self,
        replaced_id: &uuid::Uuid,
    ) -> Result<Option<Supersession>, StorageError> {
        let result = self.connection.query_row(
            "SELECT replacement_id, reason FROM supersessions WHERE replaced_id = ?1",
            params![replaced_id.to_string()],
            |row| {
                let replacement_id: String = row.get(0)?;
                let reason: String = row.get(1)?;
                Ok((replacement_id, reason))
            },
        );
        match result {
            Ok((replacement_id, reason)) => Ok(Some(Supersession {
                replaced_id: *replaced_id,
                replacement_id: uuid::Uuid::parse_str(&replacement_id)
                    .map_err(|error| StorageError::InvalidMemory(error.to_string()))?,
                reason,
            })),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(StorageError::Sqlite(error)),
        }
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
