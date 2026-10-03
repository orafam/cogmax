use cogmax_domain::{memory::MemoryKind, scope::MemoryScope};

use super::{CandidateExtractor, ExtractionError, PipelineOutcome, RawEvent};

struct FixedModel;

impl CandidateExtractor for FixedModel {
    fn extract(&self, event: &RawEvent) -> Result<super::ExtractedCandidate, ExtractionError> {
        Ok(super::ExtractedCandidate {
            scope: event.scope.clone(),
            kind: MemoryKind::Correction,
            content: "responder em português".into(),
        })
    }
}

#[test]
fn extraction_is_idempotent_by_event_id() {
    let pipeline = super::Pipeline::new(FixedModel);
    let event = RawEvent {
        id: "event-1".into(),
        scope: MemoryScope::new("user:alice").unwrap(),
        content: "preferência".into(),
    };

    assert_eq!(pipeline.process(&event), PipelineOutcome::Accepted);
    assert_eq!(pipeline.process(&event), PipelineOutcome::Duplicate);
}

#[test]
fn malformed_model_output_is_terminal() {
    struct EmptyModel;
    impl CandidateExtractor for EmptyModel {
        fn extract(&self, event: &RawEvent) -> Result<super::ExtractedCandidate, ExtractionError> {
            Ok(super::ExtractedCandidate {
                scope: event.scope.clone(),
                kind: MemoryKind::Fact,
                content: "".into(),
            })
        }
    }
    let pipeline = super::Pipeline::new(EmptyModel);
    let event = RawEvent {
        id: "event-2".into(),
        scope: MemoryScope::new("user:alice").unwrap(),
        content: "x".into(),
    };
    assert_eq!(
        pipeline.process(&event),
        PipelineOutcome::DeadLetter("empty candidate".into())
    );
}
