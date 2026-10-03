use std::collections::HashSet;
use std::sync::Mutex;

use cogmax_domain::{memory::MemoryKind, scope::MemoryScope};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct RawEvent {
    pub id: String,
    pub scope: MemoryScope,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct ExtractedCandidate {
    pub scope: MemoryScope,
    pub kind: MemoryKind,
    pub content: String,
}

#[derive(Debug, Error)]
pub enum ExtractionError {
    #[error("model unavailable: {0}")]
    Unavailable(String),
    #[error("invalid model output: {0}")]
    Invalid(String),
}

pub trait CandidateExtractor {
    fn extract(&self, event: &RawEvent) -> Result<ExtractedCandidate, ExtractionError>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum PipelineOutcome {
    Accepted,
    Duplicate,
    Retry(String),
    DeadLetter(String),
}

pub struct Pipeline<M> {
    model: M,
    processed: Mutex<HashSet<String>>,
}

impl<M: CandidateExtractor> Pipeline<M> {
    pub fn new(model: M) -> Self {
        Self {
            model,
            processed: Mutex::new(HashSet::new()),
        }
    }

    pub fn process(&self, event: &RawEvent) -> PipelineOutcome {
        {
            let mut processed = self.processed.lock().expect("pipeline mutex poisoned");
            if !processed.insert(event.id.clone()) {
                return PipelineOutcome::Duplicate;
            }
        }

        match self.model.extract(event) {
            Ok(candidate) if candidate.content.trim().is_empty() => {
                PipelineOutcome::DeadLetter("empty candidate".into())
            }
            Ok(candidate) if candidate.scope != event.scope => {
                PipelineOutcome::DeadLetter("candidate scope mismatch".into())
            }
            Ok(_) => PipelineOutcome::Accepted,
            Err(ExtractionError::Unavailable(reason)) => PipelineOutcome::Retry(reason),
            Err(ExtractionError::Invalid(reason)) => PipelineOutcome::DeadLetter(reason),
        }
    }
}

#[cfg(test)]
mod tests;
