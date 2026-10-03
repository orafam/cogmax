use std::sync::{Arc, Mutex};

use cogmax_domain::{
    candidate::MemoryCandidate,
    memory::{Authority, Confidence, Memory, MemoryStatus},
    scope::MemoryScope,
};
use cogmax_storage::{SqliteStore, StorageError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("storage error: {0}")]
    Storage(#[from] StorageError),
}

#[derive(Debug, Clone)]
pub struct RecallRequest {
    pub scope: MemoryScope,
    pub query: String,
    pub kind: Option<cogmax_domain::memory::MemoryKind>,
    pub project: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LearnRequest {
    pub candidate: MemoryCandidate,
    pub confidence: Confidence,
    pub authority: Authority,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecallMatch {
    pub memory: Memory,
    pub reasons: Vec<String>,
}

pub struct MemoryService {
    store: Arc<Mutex<SqliteStore>>,
}

#[cfg(test)]
mod tests;

impl MemoryService {
    pub fn new(store: Arc<Mutex<SqliteStore>>) -> Self {
        Self { store }
    }

    pub fn recall(&self, request: RecallRequest) -> Result<Vec<Memory>, CoreError> {
        let terms = request
            .query
            .to_lowercase()
            .split_whitespace()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        let store = self.store.lock().expect("storage mutex poisoned");
        let mut memories = match request.project.as_deref() {
            Some(project) => store.list_active_prefix(&format!(
                "{}/project:{}",
                request.scope.as_str(),
                project
            ))?,
            None => store.list_active(&request.scope)?,
        };
        if let Some(kind) = request.kind {
            memories.retain(|memory| memory.kind == kind);
        }
        memories.retain(|memory| {
            terms.is_empty()
                || terms
                    .iter()
                    .any(|term| memory.content.to_lowercase().contains(term))
        });
        memories.sort_by_key(|memory| {
            let content = memory.content.to_lowercase();
            let matched_terms = terms
                .iter()
                .filter(|term| content.contains(term.as_str()))
                .count();
            let kind_priority = match memory.kind {
                cogmax_domain::memory::MemoryKind::Decision => 3,
                cogmax_domain::memory::MemoryKind::Project => 2,
                cogmax_domain::memory::MemoryKind::Procedure => 1,
                _ => 0,
            };
            (
                std::cmp::Reverse(matched_terms),
                std::cmp::Reverse(kind_priority),
                memory.id,
            )
        });
        Ok(memories)
    }

    pub fn recall_explained(&self, request: RecallRequest) -> Result<Vec<RecallMatch>, CoreError> {
        let query = request.query.clone();
        let project = request.project.clone();
        let kind = request.kind;
        self.recall(request)?
            .into_iter()
            .map(|memory| {
                let lower = memory.content.to_lowercase();
                let matched = query
                    .to_lowercase()
                    .split_whitespace()
                    .filter(|term| lower.contains(*term))
                    .count();
                let mut reasons = Vec::new();
                if query.trim().is_empty() {
                    reasons.push("consulta sem filtro textual".into());
                } else {
                    reasons.push(format!("{} termos da consulta correspondidos", matched));
                }
                if project.is_some() {
                    reasons.push("escopo de projeto aplicado".into());
                }
                if let Some(kind) = kind {
                    reasons.push(format!("tipo {:?} solicitado", kind));
                }
                reasons.push(format!("autoridade {:?}", memory.authority));
                reasons.push(format!("confiança {:?}", memory.confidence));
                Ok(RecallMatch { memory, reasons })
            })
            .collect()
    }

    pub fn learn(&self, request: LearnRequest) -> Result<bool, CoreError> {
        if request.confidence == Confidence::Low {
            return Ok(false);
        }
        let store = self.store.lock().expect("storage mutex poisoned");
        let inserted = store.insert_candidate(&request.candidate)?;
        if inserted {
            let memory = Memory::new(
                request.candidate.scope,
                request.candidate.kind,
                request.candidate.content,
                request.confidence,
                request.authority,
            );
            store.insert_memory(&memory)?;
        }
        Ok(inserted)
    }

    pub fn forget(&self, id: &uuid::Uuid) -> Result<(), CoreError> {
        self.store
            .lock()
            .expect("storage mutex poisoned")
            .set_status(id, MemoryStatus::Revoked)?;
        Ok(())
    }

    pub fn supersede(&self, id: &uuid::Uuid) -> Result<(), CoreError> {
        self.store
            .lock()
            .expect("storage mutex poisoned")
            .set_status(id, MemoryStatus::Superseded)?;
        Ok(())
    }
}
