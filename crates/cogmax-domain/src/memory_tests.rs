use super::candidate::MemoryCandidate;
use super::memory::{Authority, Confidence, Memory, MemoryKind, MemoryStatus};
use super::scope::MemoryScope;

#[test]
fn memory_serializes_and_round_trips_with_stable_identity() {
    let scope = MemoryScope::new("user:alice").unwrap();
    let memory = Memory::new(
        scope,
        MemoryKind::Preference,
        "Responder em português".into(),
        Confidence::High,
        Authority::Explicit,
    );

    let encoded = serde_json::to_string(&memory).unwrap();
    let decoded: Memory = serde_json::from_str(&encoded).unwrap();

    assert_eq!(memory, decoded);
    assert_eq!(memory.status, MemoryStatus::Active);
    assert_eq!(memory.id, decoded.id);
}

#[test]
fn memory_scope_rejects_empty_or_malformed_values() {
    assert!(MemoryScope::new("").is_err());
    assert!(MemoryScope::new("user alice").is_err());
    assert!(MemoryScope::new("agent:codex/project:rrc").is_ok());
}

#[test]
fn candidate_has_stable_idempotency_key() {
    let candidate = MemoryCandidate::new(
        "event-123".into(),
        MemoryScope::new("user:alice").unwrap(),
        MemoryKind::Correction,
        "Responder em português".into(),
    );
    let duplicate = candidate.clone();

    assert_eq!(candidate.idempotency_key(), duplicate.idempotency_key());
}
