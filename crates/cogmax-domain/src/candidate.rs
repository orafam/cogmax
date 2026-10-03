use crate::{memory::MemoryKind, scope::MemoryScope};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryCandidate {
    pub source_event_id: String,
    pub scope: MemoryScope,
    pub kind: MemoryKind,
    pub content: String,
}

impl MemoryCandidate {
    pub fn new(
        source_event_id: String,
        scope: MemoryScope,
        kind: MemoryKind,
        content: String,
    ) -> Self {
        Self {
            source_event_id,
            scope,
            kind,
            content,
        }
    }

    pub fn idempotency_key(&self) -> String {
        format!(
            "{}:{}:{:?}:{}",
            self.source_event_id,
            self.scope.as_str(),
            self.kind,
            self.content
        )
    }
}
