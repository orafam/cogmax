use std::sync::Arc;

use arrow_array::{ArrayRef, RecordBatch, StringArray};
use arrow_schema::{DataType, Field, Schema};
use parquet::arrow::ArrowWriter;
use sha2::{Digest, Sha256};
use cogmax_domain::memory::Memory;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("arrow error: {0}")]
    Arrow(#[from] arrow_schema::ArrowError),
    #[error("parquet error: {0}")]
    Parquet(#[from] parquet::errors::ParquetError),
    #[error("git synchronization failed: {0}")]
    Git(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotManifest {
    pub schema_version: u32,
    pub memory_count: usize,
    pub sha256: String,
}

pub fn markdown(memories: &[Memory]) -> String {
    let mut ordered = memories.to_vec();
    ordered.sort_by_key(|memory| memory.id);
    let mut output = String::from("# Cogmax Memory Snapshot\n\n");
    for memory in ordered {
        output.push_str(&format!(
            "## {}\n\n- scope: `{}`\n- kind: `{:?}`\n- status: `{:?}`\n\n{}\n\n",
            memory.id,
            memory.scope.as_str(),
            memory.kind,
            memory.status,
            memory.content
        ));
    }
    output
}

pub fn parquet(memories: &[Memory]) -> Result<Vec<u8>, ExportError> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new("scope", DataType::Utf8, false),
        Field::new("content", DataType::Utf8, false),
    ]));
    let ids: ArrayRef = Arc::new(StringArray::from_iter_values(
        memories.iter().map(|m| m.id.to_string()),
    ));
    let scopes: ArrayRef = Arc::new(StringArray::from_iter_values(
        memories.iter().map(|m| m.scope.as_str()),
    ));
    let contents: ArrayRef = Arc::new(StringArray::from_iter_values(
        memories.iter().map(|m| m.content.as_str()),
    ));
    let batch = RecordBatch::try_new(schema.clone(), vec![ids, scopes, contents])?;
    let mut bytes = Vec::new();
    {
        let mut writer = ArrowWriter::try_new(&mut bytes, schema, None)?;
        writer.write(&batch)?;
        writer.close()?;
    }
    Ok(bytes)
}

pub fn manifest(markdown: &str, memory_count: usize) -> SnapshotManifest {
    let mut hasher = Sha256::new();
    hasher.update(markdown.as_bytes());
    SnapshotManifest {
        schema_version: 1,
        memory_count,
        sha256: format!("{:x}", hasher.finalize()),
    }
}

pub trait GitSync {
    fn publish(&self, markdown: &str, manifest: &SnapshotManifest) -> Result<(), ExportError>;
}

#[cfg(test)]
mod tests;
