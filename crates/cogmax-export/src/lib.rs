use std::sync::Arc;

use arrow_array::{ArrayRef, RecordBatch, StringArray};
use arrow_schema::{DataType, Field, Schema};
use cogmax_domain::memory::Memory;
use parquet::arrow::ArrowWriter;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

pub mod git;

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("arrow error: {0}")]
    Arrow(#[from] arrow_schema::ArrowError),
    #[error("parquet error: {0}")]
    Parquet(#[from] parquet::errors::ParquetError),
    #[error("git synchronization failed: {0}")]
    Git(String),
    #[error("snapshot manifest does not match its content")]
    ManifestMismatch,
    #[error("filesystem error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    let mut ordered = memories.to_vec();
    ordered.sort_by_key(|memory| memory.id);
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new("scope", DataType::Utf8, false),
        Field::new("kind", DataType::Utf8, false),
        Field::new("status", DataType::Utf8, false),
        Field::new("confidence", DataType::Utf8, false),
        Field::new("authority", DataType::Utf8, false),
        Field::new("content", DataType::Utf8, false),
    ]));
    let ids: ArrayRef = Arc::new(StringArray::from_iter_values(
        ordered.iter().map(|m| m.id.to_string()),
    ));
    let scopes: ArrayRef = Arc::new(StringArray::from_iter_values(
        ordered.iter().map(|m| m.scope.as_str()),
    ));
    let kinds: ArrayRef = Arc::new(StringArray::from_iter_values(
        ordered.iter().map(|m| format!("{:?}", m.kind)),
    ));
    let statuses: ArrayRef = Arc::new(StringArray::from_iter_values(
        ordered.iter().map(|m| format!("{:?}", m.status)),
    ));
    let confidences: ArrayRef = Arc::new(StringArray::from_iter_values(
        ordered.iter().map(|m| format!("{:?}", m.confidence)),
    ));
    let authorities: ArrayRef = Arc::new(StringArray::from_iter_values(
        ordered.iter().map(|m| format!("{:?}", m.authority)),
    ));
    let contents: ArrayRef = Arc::new(StringArray::from_iter_values(
        ordered.iter().map(|m| m.content.as_str()),
    ));
    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            ids,
            scopes,
            kinds,
            statuses,
            confidences,
            authorities,
            contents,
        ],
    )?;
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

pub fn verify_manifest(markdown: &str, expected: &SnapshotManifest) -> Result<(), ExportError> {
    let actual = manifest(markdown, markdown_memory_count(markdown));
    if actual == *expected {
        Ok(())
    } else {
        Err(ExportError::ManifestMismatch)
    }
}

fn markdown_memory_count(markdown: &str) -> usize {
    markdown
        .lines()
        .filter(|line| line.starts_with("## "))
        .count()
}

pub trait GitSync {
    fn publish(&self, markdown: &str, manifest: &SnapshotManifest) -> Result<(), ExportError>;
}

#[cfg(test)]
mod tests;
