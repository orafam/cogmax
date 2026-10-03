use crate::scope::MemoryScope;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryKind {
    Preference,
    Fact,
    Project,
    Procedure,
    Decision,
    Correction,
    Reference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryStatus {
    Active,
    Superseded,
    Revoked,
    Expired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Confidence {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Authority {
    Inferred,
    Explicit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Memory {
    pub id: Uuid,
    pub scope: MemoryScope,
    pub kind: MemoryKind,
    pub content: String,
    pub confidence: Confidence,
    pub authority: Authority,
    pub status: MemoryStatus,
}

impl Memory {
    pub fn new(
        scope: MemoryScope,
        kind: MemoryKind,
        content: String,
        confidence: Confidence,
        authority: Authority,
    ) -> Self {
        let seed = format!("{}:{kind:?}:{content}", scope.as_str());
        Self {
            id: Uuid::new_v5(&Uuid::NAMESPACE_OID, seed.as_bytes()),
            scope,
            kind,
            content,
            confidence,
            authority,
            status: MemoryStatus::Active,
        }
    }
}
