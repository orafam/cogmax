use std::sync::{Arc, Mutex};

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use cogmax_core::{LearnRequest, MemoryService, RecallRequest};
use cogmax_domain::{
    candidate::MemoryCandidate,
    memory::{Authority, Confidence, MemoryKind},
    scope::MemoryScope,
};
use cogmax_storage::SqliteStore;

#[derive(Clone)]
pub struct ApiState {
    pub service: Arc<MemoryService>,
}

#[derive(Debug, Deserialize)]
pub struct RecallInput {
    pub scope: String,
    pub query: String,
    pub kind: Option<MemoryKind>,
    pub project: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RecallOutput {
    pub memories: Vec<cogmax_domain::memory::Memory>,
}

#[derive(Debug, Deserialize)]
pub struct LearnInput {
    pub event_id: String,
    pub scope: String,
    pub kind: MemoryKind,
    pub content: String,
    pub confidence: Confidence,
    pub authority: Authority,
}

pub fn router(service: Arc<MemoryService>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/recall", post(recall))
        .route("/learn", post(learn))
        .with_state(ApiState { service })
}

async fn health() -> &'static str {
    "ok"
}

async fn recall(
    State(state): State<ApiState>,
    Json(input): Json<RecallInput>,
) -> Json<RecallOutput> {
    let scope = MemoryScope::new(input.scope).expect("invalid scope");
    let memories = state
        .service
        .recall(RecallRequest {
            scope,
            query: input.query,
            kind: input.kind,
            project: input.project,
        })
        .expect("recall failed");
    Json(RecallOutput { memories })
}

async fn learn(State(state): State<ApiState>, Json(input): Json<LearnInput>) -> Json<bool> {
    let candidate = MemoryCandidate::new(
        input.event_id,
        MemoryScope::new(input.scope).expect("invalid scope"),
        input.kind,
        input.content,
    );
    let result = state
        .service
        .learn(LearnRequest {
            candidate,
            confidence: input.confidence,
            authority: input.authority,
        })
        .expect("learn failed");
    Json(result)
}

pub fn local_router() -> Router {
    let store = Arc::new(Mutex::new(SqliteStore::in_memory().expect("storage")));
    router(Arc::new(MemoryService::new(store)))
}

#[cfg(test)]
mod tests {
    #[test]
    fn local_router_is_constructible() {
        let _ = super::local_router();
    }
}
