use std::sync::{Arc, Mutex};

use cogmax_domain::{
    candidate::MemoryCandidate,
    memory::{Authority, Confidence, MemoryKind},
    scope::MemoryScope,
};
use cogmax_storage::SqliteStore;

use super::{LearnRequest, MemoryService, RecallRequest};

#[test]
fn explicit_learning_is_recalled_in_a_later_request() {
    let service = MemoryService::new(Arc::new(Mutex::new(SqliteStore::in_memory().unwrap())));
    let scope = MemoryScope::new("user:alice").unwrap();
    let candidate = MemoryCandidate::new(
        "event-1".into(),
        scope.clone(),
        MemoryKind::Correction,
        "responder em português".into(),
    );

    assert!(service
        .learn(LearnRequest {
            candidate,
            confidence: Confidence::High,
            authority: Authority::Explicit,
        })
        .unwrap());
    let result = service
        .recall(RecallRequest {
            scope,
            query: "português".into(),
            kind: None,
            project: None,
        })
        .unwrap();
    assert_eq!(result.len(), 1);
}

#[test]
fn low_confidence_learning_is_not_activated() {
    let service = MemoryService::new(Arc::new(Mutex::new(SqliteStore::in_memory().unwrap())));
    let scope = MemoryScope::new("user:alice").unwrap();
    let candidate = MemoryCandidate::new(
        "event-2".into(),
        scope.clone(),
        MemoryKind::Fact,
        "gosta de chá".into(),
    );

    assert!(!service
        .learn(LearnRequest {
            candidate,
            confidence: Confidence::Low,
            authority: Authority::Inferred,
        })
        .unwrap());
    assert!(service
        .recall(RecallRequest {
            scope,
            query: "chá".into(),
            kind: None,
            project: None,
        })
        .unwrap()
        .is_empty());
}

#[test]
fn recall_orders_decisions_and_projects_before_references() {
    let service = MemoryService::new(Arc::new(Mutex::new(SqliteStore::in_memory().unwrap())));
    let scope = MemoryScope::new("user:alice").unwrap();
    for (event, kind, content) in [
        ("reference", MemoryKind::Reference, "Cogmax usa SQLite"),
        ("project", MemoryKind::Project, "Projeto Cogmax usa SQLite"),
        ("decision", MemoryKind::Decision, "Decisão: Cogmax usa SQLite"),
    ] {
        assert!(service
            .learn(LearnRequest {
                candidate: MemoryCandidate::new(event.into(), scope.clone(), kind, content.into(),),
                confidence: Confidence::High,
                authority: Authority::Explicit,
            })
            .unwrap());
    }

    let result = service
        .recall(RecallRequest {
            scope,
            query: "Cogmax SQLite".into(),
            kind: None,
            project: None,
        })
        .unwrap();
    assert_eq!(result[0].kind, MemoryKind::Decision);
    assert_eq!(result[1].kind, MemoryKind::Project);
    assert_eq!(result[2].kind, MemoryKind::Reference);
}

#[test]
fn recall_can_filter_by_kind_and_project_scope() {
    let service = MemoryService::new(Arc::new(Mutex::new(SqliteStore::in_memory().unwrap())));
    let user_scope = MemoryScope::new("user:alice").unwrap();
    let project_scope = MemoryScope::new("user:alice/project:cogmax").unwrap();
    for (event, scope, kind, content) in [
        (
            "reference",
            user_scope.clone(),
            MemoryKind::Reference,
            "SQLite",
        ),
        (
            "project",
            project_scope.clone(),
            MemoryKind::Project,
            "SQLite",
        ),
    ] {
        assert!(service
            .learn(LearnRequest {
                candidate: MemoryCandidate::new(event.into(), scope, kind, content.into()),
                confidence: Confidence::High,
                authority: Authority::Explicit,
            })
            .unwrap());
    }

    let result = service
        .recall(RecallRequest {
            scope: user_scope,
            query: "SQLite".into(),
            kind: Some(MemoryKind::Project),
            project: Some("cogmax".into()),
        })
        .unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].kind, MemoryKind::Project);
    assert_eq!(result[0].scope, project_scope);
}
