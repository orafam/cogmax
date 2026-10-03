use rusqlite::Connection;
use cogmax_domain::{
    candidate::MemoryCandidate,
    memory::{Authority, Confidence, Memory, MemoryKind, MemoryStatus},
    scope::MemoryScope,
};

use super::SqliteStore;

fn store() -> SqliteStore {
    SqliteStore::open(Connection::open_in_memory().unwrap()).unwrap()
}

#[test]
fn candidate_is_idempotent_and_consolidates_once() {
    let store = store();
    let candidate = MemoryCandidate::new(
        "event-1".into(),
        MemoryScope::new("user:alice").unwrap(),
        MemoryKind::Correction,
        "Responder em português".into(),
    );

    assert!(store.insert_candidate(&candidate).unwrap());
    assert!(!store.insert_candidate(&candidate).unwrap());
    let memory = Memory::new(
        candidate.scope.clone(),
        candidate.kind,
        candidate.content.clone(),
        Confidence::High,
        Authority::Explicit,
    );
    store.insert_memory(&memory).unwrap();
    assert_eq!(store.list_active(&candidate.scope).unwrap(), vec![memory]);
}

#[test]
fn recall_isolated_by_scope_and_ignores_inactive_memories() {
    let store = store();
    let alice = Memory::new(
        MemoryScope::new("user:alice").unwrap(),
        MemoryKind::Preference,
        "café".into(),
        Confidence::High,
        Authority::Explicit,
    );
    let bob = Memory::new(
        MemoryScope::new("user:bob").unwrap(),
        MemoryKind::Preference,
        "chá".into(),
        Confidence::High,
        Authority::Explicit,
    );
    store.insert_memory(&alice).unwrap();
    store.insert_memory(&bob).unwrap();
    store.set_status(&alice.id, MemoryStatus::Expired).unwrap();

    assert!(store.list_active(&alice.scope).unwrap().is_empty());
    assert_eq!(store.list_active(&bob.scope).unwrap(), vec![bob]);
}

#[test]
fn rebuild_import_removes_only_inferred_memories() {
    let store = store();
    let scope = MemoryScope::new("user:local").unwrap();
    let inferred = Memory::new(
        scope.clone(),
        MemoryKind::Reference,
        "importada".into(),
        Confidence::Medium,
        Authority::Inferred,
    );
    let explicit = Memory::new(
        scope.clone(),
        MemoryKind::Preference,
        "explícita".into(),
        Confidence::High,
        Authority::Explicit,
    );
    store.insert_memory(&inferred).unwrap();
    store.insert_memory(&explicit).unwrap();
    store
        .insert_candidate(&MemoryCandidate::new(
            "imported-event".into(),
            scope.clone(),
            MemoryKind::Reference,
            "importada".into(),
        ))
        .unwrap();

    assert_eq!(store.reset_inferred_imports().unwrap(), 1);
    assert_eq!(store.list_active(&scope).unwrap(), vec![explicit]);
    assert!(store
        .insert_candidate(&MemoryCandidate::new(
            "imported-event".into(),
            scope,
            MemoryKind::Reference,
            "importada".into(),
        ))
        .unwrap());
}
