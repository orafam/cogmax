use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use axum::{
    extract::State,
    http::HeaderMap,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use cogmax_core::{LearnRequest, MemoryService, RecallRequest};
use cogmax_domain::{
    candidate::MemoryCandidate,
    memory::{Authority, Confidence, MemoryKind},
    scope::MemoryScope,
};
use cogmax_storage::SqliteStore;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct ApiState {
    pub service: Arc<MemoryService>,
    pub api_token: Option<String>,
    pub api_user: Option<String>,
    rate_limit: Option<Arc<Mutex<RateWindow>>>,
}

struct RateWindow {
    started: Instant,
    requests: u32,
    max_requests: u32,
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
    pub explanations: Vec<RecallExplanation>,
}

#[derive(Debug, Serialize)]
pub struct RecallExplanation {
    pub memory_id: String,
    pub reasons: Vec<String>,
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

#[derive(Debug, Deserialize)]
pub struct SupersedeInput {
    pub replaced_id: String,
    pub replacement_id: String,
    pub reason: String,
}

pub fn router(service: Arc<MemoryService>) -> Router {
    router_with_identity(
        service,
        std::env::var("COGMAX_API_TOKEN").ok(),
        std::env::var("COGMAX_API_USER").ok(),
    )
}

pub fn router_with_token(service: Arc<MemoryService>, api_token: Option<String>) -> Router {
    router_with_identity(service, api_token, None)
}

pub fn router_with_identity(
    service: Arc<MemoryService>,
    api_token: Option<String>,
    api_user: Option<String>,
) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/recall", post(recall))
        .route("/learn", post(learn))
        .route("/supersede", post(supersede))
        .route("/revoke", post(revoke))
        .with_state(ApiState {
            service,
            api_token,
            api_user,
            rate_limit: Some(Arc::new(Mutex::new(RateWindow {
                started: Instant::now(),
                requests: 0,
                max_requests: std::env::var("COGMAX_RATE_LIMIT")
                    .ok()
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(120),
            }))),
        })
}

async fn health() -> &'static str {
    "ok"
}

async fn recall(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Json(input): Json<RecallInput>,
) -> Result<Json<RecallOutput>, StatusCode> {
    authorize(&headers, &state)?;
    let scope = MemoryScope::new(input.scope).map_err(|_| StatusCode::BAD_REQUEST)?;
    authorize_scope(&scope, input.project.as_deref(), &state)?;
    let matches = state
        .service
        .recall_explained(RecallRequest {
            scope,
            query: input.query,
            kind: input.kind,
            project: input.project,
        })
        .expect("recall failed");
    let explanations = matches
        .iter()
        .map(|matched| RecallExplanation {
            memory_id: matched.memory.id.to_string(),
            reasons: matched.reasons.clone(),
        })
        .collect();
    let memories = matches.into_iter().map(|matched| matched.memory).collect();
    Ok(Json(RecallOutput {
        memories,
        explanations,
    }))
}

async fn learn(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Json(input): Json<LearnInput>,
) -> Result<Json<bool>, StatusCode> {
    authorize(&headers, &state)?;
    let scope = MemoryScope::new(input.scope).map_err(|_| StatusCode::BAD_REQUEST)?;
    authorize_scope(&scope, None, &state)?;
    if input.authority != Authority::Explicit {
        return Err(StatusCode::FORBIDDEN);
    }
    let candidate = MemoryCandidate::new(input.event_id, scope, input.kind, input.content);
    let result = state
        .service
        .learn(LearnRequest {
            candidate,
            confidence: input.confidence,
            authority: input.authority,
        })
        .expect("learn failed");
    Ok(Json(result))
}

async fn supersede(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Json(input): Json<SupersedeInput>,
) -> Result<Json<bool>, StatusCode> {
    authorize(&headers, &state)?;
    let replaced_id =
        uuid::Uuid::parse_str(&input.replaced_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let replacement_id =
        uuid::Uuid::parse_str(&input.replacement_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    if input.reason.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    authorize_memory_scope(&state, &replaced_id)?;
    authorize_memory_scope(&state, &replacement_id)?;
    state
        .service
        .supersede(&replaced_id, &replacement_id, &input.reason)
        .map_err(|_| StatusCode::CONFLICT)?;
    Ok(Json(true))
}

async fn revoke(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Json(input): Json<IdInput>,
) -> Result<Json<bool>, StatusCode> {
    authorize(&headers, &state)?;
    let id = uuid::Uuid::parse_str(&input.id).map_err(|_| StatusCode::BAD_REQUEST)?;
    authorize_memory_scope(&state, &id)?;
    state
        .service
        .forget(&id)
        .map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(Json(true))
}

#[derive(Debug, Deserialize)]
pub struct IdInput {
    pub id: String,
}

fn authorize(headers: &HeaderMap, state: &ApiState) -> Result<(), StatusCode> {
    let Some(expected) = state.api_token.as_deref() else {
        return Ok(());
    };
    let provided = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));
    if provided != Some(expected) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let Some(rate_limit) = &state.rate_limit else {
        return Ok(());
    };
    let mut window = rate_limit.lock().expect("rate limiter mutex poisoned");
    if window.started.elapsed() >= Duration::from_secs(60) {
        window.started = Instant::now();
        window.requests = 0;
    }
    if window.requests >= window.max_requests {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }
    window.requests += 1;
    Ok(())
}

fn authorize_scope(
    scope: &MemoryScope,
    project: Option<&str>,
    state: &ApiState,
) -> Result<(), StatusCode> {
    let Some(user) = state.api_user.as_deref() else {
        return Ok(());
    };
    let expected = format!("user:{user}");
    let valid_scope = scope.as_str() == expected
        || scope
            .as_str()
            .strip_prefix(&expected)
            .is_some_and(|suffix| suffix.starts_with("/project:"));
    if !valid_scope {
        return Err(StatusCode::FORBIDDEN);
    }
    if project.is_some_and(|project| project.is_empty() || project.contains('/')) {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(())
}

fn authorize_memory_scope(state: &ApiState, id: &uuid::Uuid) -> Result<(), StatusCode> {
    let scope = state
        .service
        .scope_for(id)
        .map_err(|_| StatusCode::NOT_FOUND)?;
    authorize_scope(&scope, None, state)
}

pub fn local_router() -> Router {
    let store = Arc::new(Mutex::new(SqliteStore::in_memory().expect("storage")));
    router_with_token(Arc::new(MemoryService::new(store)), None)
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use axum::{body::Body, http::Request};
    use cogmax_core::MemoryService;
    use cogmax_storage::SqliteStore;
    use tower::ServiceExt;

    use super::{authorize_scope, router_with_identity, router_with_token, ApiState};

    #[test]
    fn local_router_is_constructible() {
        let _ = super::local_router();
    }

    #[tokio::test]
    async fn protected_routes_fail_closed_without_bearer_token() {
        let store = Arc::new(Mutex::new(SqliteStore::in_memory().unwrap()));
        let app = router_with_token(
            Arc::new(MemoryService::new(store)),
            Some("test-token".into()),
        );
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/recall")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"scope":"user:alice","query":""}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn supersede_rejects_invalid_uuid_after_authentication() {
        let store = Arc::new(Mutex::new(SqliteStore::in_memory().unwrap()));
        let app = router_with_token(
            Arc::new(MemoryService::new(store)),
            Some("test-token".into()),
        );
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/supersede")
                    .method("POST")
                    .header("authorization", "Bearer test-token")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"replaced_id":"bad","replacement_id":"bad","reason":"x"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
    }

    #[test]
    fn identity_cannot_cross_user_or_project_boundary() {
        let store = Arc::new(Mutex::new(SqliteStore::in_memory().unwrap()));
        let service = Arc::new(MemoryService::new(store));
        let app = router_with_identity(service.clone(), Some("token".into()), Some("alice".into()));
        let state = ApiState {
            service,
            api_token: Some("token".into()),
            api_user: Some("alice".into()),
            rate_limit: None,
        };
        assert!(authorize_scope(
            &cogmax_domain::scope::MemoryScope::new("user:alice/project:cogmax").unwrap(),
            Some("cogmax"),
            &state
        )
        .is_ok());
        assert_eq!(
            authorize_scope(
                &cogmax_domain::scope::MemoryScope::new("user:bob").unwrap(),
                None,
                &state
            ),
            Err(axum::http::StatusCode::FORBIDDEN)
        );
        assert_eq!(
            authorize_scope(
                &cogmax_domain::scope::MemoryScope::new("user:alice").unwrap(),
                Some("other/project"),
                &state
            ),
            Err(axum::http::StatusCode::BAD_REQUEST)
        );
        let _ = app;
    }
}
