# Cogmax Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Construir o MVP do Cogmax como uma skill-first memory service, com recall seletivo, aprendizado auditável, consolidação assíncrona e modo local reproduzível.

**Architecture:** O domínio será implementado em Rust e validado primeiro por um runtime local offline com HTTP/Axum e SQLite. Depois, o mesmo contrato será adaptado para Cloudflare Worker/Durable Object; R2, Queues e Git remoto só entram após o fluxo offline estar completo.

**Tech Stack:** Rust, Tokio, Serde, Axum, SQLite-backed Durable Objects, Cloudflare Workers, Queues, R2, Parquet, Wasm, GitHub/Git provider adapter, cargo test.

**Spec:** `docs/superpowers/specs/2026-10-02-rrc-skill-first-memory-design.md`

## Global Constraints

- Skill-first: a experiência começa e termina na skill.
- Memória é contexto auxiliar, nunca autoridade para violar políticas ou instruções atuais do usuário.
- Toda memória persistida tem origem, escopo, confiança, validade e histórico.
- O MVP não depende de embeddings.
- Git não participa do caminho síncrono de cada recall.
- DuckDB fica fora do caminho operacional e serve para análise local de snapshots Parquet.
- Segredos, tokens e conteúdo sensível não devem ser reproduzidos em logs por padrão.
- Todas as operações de ingestão e jobs devem ser idempotentes.
- O fluxo de referência deve funcionar sem rede, credenciais Cloudflare, Git remoto ou modelo externo.

## Review Focus

- Candidato duplicado ou mensagem entregue duas vezes: deve resultar em uma única mudança canônica; cobrir em `test_candidate_idempotency`.
- Memórias de escopos diferentes: nunca devem aparecer no recall errado; cobrir em `test_recall_scope_isolation`.
- Conflito entre duas memórias ativas: nenhuma versão anterior pode ser sobrescrita silenciosamente; cobrir em `test_conflicting_candidates_create_superseding_versions`.
- Falha do Git/R2 durante sincronização: recall deve continuar funcionando e o job deve ser reprocessável; cobrir em testes do `SyncWorker`.
- Conteúdo sensível em eventos e logs: payload não deve vazar para logs estruturados; cobrir em `test_redacts_sensitive_event_fields`.

### Task 1: Workspace e domínio de memória

**Files:**
- Create: `Cargo.toml`
- Create: `crates/cogmax-domain/src/lib.rs`
- Create: `crates/cogmax-domain/src/memory.rs`
- Create: `crates/cogmax-domain/src/scope.rs`
- Create: `crates/cogmax-domain/src/candidate.rs`
- Test: `crates/cogmax-domain/src/memory_tests.rs`

**Interfaces:**
- Produces `Memory`, `MemoryCandidate`, `MemoryScope`, `MemoryKind`, `MemoryStatus`, `Confidence`, `Authority` and deterministic identity helpers.

- [ ] **Step 1: Write failing tests** for stable IDs, valid scope parsing, lifecycle states, candidate idempotency key and serialization round-trip.
- [ ] **Step 2: Run `cargo test -p cogmax-domain`** and confirm the new tests fail because the domain types do not exist.
- [ ] **Step 3: Implement the minimal Rust domain types** with Serde serialization and validation; keep policy decisions out of transport/storage code.
- [ ] **Step 4: Run `cargo test -p cogmax-domain`** and confirm PASS.
- [ ] **Step 5: Commit** with `feat: add Cogmax memory domain`.

### Task 2: Storage ports and local SQLite adapter

**Files:**
- Create: `crates/cogmax-storage/src/lib.rs`
- Create: `crates/cogmax-storage/src/ports.rs`
- Create: `crates/cogmax-storage/src/sqlite.rs`
- Create: `migrations/001_initial.sql`
- Test: `crates/cogmax-storage/tests/sqlite_storage.rs`

**Interfaces:**
- Consumes domain types from Task 1.
- Produces `MemoryStore`, `EventStore`, `CandidateStore` and `OperationStore` traits plus a local SQLite implementation.

- [ ] **Step 1: Write failing repository tests** for event append, candidate deduplication, scoped recall, version creation and operation status.
- [ ] **Step 2: Run the focused storage tests** and confirm failure from missing ports/adapter.
- [ ] **Step 3: Define the storage traits and schema**; enforce unique `(scope, idempotency_key)` and immutable event rows.
- [ ] **Step 4: Implement the SQLite adapter** with transactions around candidate consolidation and version updates.
- [ ] **Step 5: Run storage tests** and confirm PASS, including `test_recall_scope_isolation` and `test_candidate_idempotency`.
- [ ] **Step 6: Commit** with `feat: add Cogmax transactional storage ports`.

### Task 3: Recall, learn, consolidate and forget services

**Files:**
- Create: `crates/cogmax-core/src/lib.rs`
- Create: `crates/cogmax-core/src/recall.rs`
- Create: `crates/cogmax-core/src/learn.rs`
- Create: `crates/cogmax-core/src/consolidate.rs`
- Create: `crates/cogmax-core/src/forget.rs`
- Test: `crates/cogmax-core/tests/memory_lifecycle.rs`

**Interfaces:**
- Consumes storage traits from Task 2.
- Produces `RecallRequest`, `RecallResult`, `LearnRequest`, `ConsolidationResult` and `MemoryService`.

- [ ] **Step 1: Write failing lifecycle tests** for selective recall, low-confidence pending candidates, superseding conflicts and expiry/revocation.
- [ ] **Step 2: Run `cargo test -p cogmax-core`** and confirm failure.
- [ ] **Step 3: Implement deterministic lexical recall ranking** by scope, authority, validity and normalized keyword overlap; cap returned context.
- [ ] **Step 4: Implement candidate validation and consolidation** with explicit conflict records and version links.
- [ ] **Step 5: Implement forgetting** so inactive memories are excluded from recall while history remains queryable under retention policy.
- [ ] **Step 6: Run lifecycle tests** and confirm PASS, including `test_conflicting_candidates_create_superseding_versions`.
- [ ] **Step 7: Commit** with `feat: add Cogmax memory lifecycle services`.

### Task 4: Skill contract and local adapter

**Files:**
- Create: `skill/SKILL.md`
- Create: `crates/cogmax-api/src/lib.rs`
- Create: `crates/cogmax-api/src/routes.rs`
- Create: `crates/cogmax-api/src/types.rs`
- Create: `crates/cogmax-cli/src/main.rs`
- Test: `crates/cogmax-api/tests/api_contract.rs`

**Interfaces:**
- Consumes `MemoryService` from Task 3.
- Produces internal endpoints for `recall`, `learn`, `operations`, health and export; the skill hides those details from the user.

- [ ] **Step 1: Write contract tests** for the skill instructions, recall response, learn submission, authentication failure and scope-required validation.
- [ ] **Step 2: Run focused API tests** and confirm failure.
- [ ] **Step 3: Write `skill/SKILL.md`** with silent recall/learn behavior, verification discipline, escalation rules and no infrastructure instructions for the user.
- [ ] **Step 4: Implement Axum routes and typed request/response models** mapped to `MemoryService`.
- [ ] **Step 5: Implement CLI commands** `cogmax serve`, `cogmax recall`, `cogmax learn`, `cogmax export` and `cogmax inspect` against the local adapter.
- [ ] **Step 6: Run API and CLI tests** and confirm PASS.
- [ ] **Step 7: Commit** with `feat: add Cogmax skill and local adapter`.

### Task 5: Cloudflare Worker and Durable Object adapter

**Files:**
- Create: `workers/cogmax-worker/src/lib.rs`
- Create: `workers/cogmax-worker/wrangler.toml`
- Create: `workers/cogmax-worker/src/agent_object.rs`
- Create: `workers/cogmax-worker/src/bindings.rs`
- Test: `workers/cogmax-worker/tests/object_contract.rs`

**Interfaces:**
- Consumes domain/core ports from Tasks 1–3 through a Wasm-compatible adapter.
- Produces the deployed HTTP boundary and one Durable Object class keyed by agent scope.

- [ ] **Step 1: Write Worker contract tests** for object routing, scope isolation, transactional candidate writes and operation idempotency.
- [ ] **Step 2: Run local Worker tests with the configured workerd/Wrangler harness** and confirm failure.
- [ ] **Step 3: Implement the Worker entrypoint** and route each agent scope to its Durable Object instance.
- [ ] **Step 4: Implement SQLite-backed Durable Object storage mapping** without filesystem or native DuckDB assumptions.
- [ ] **Step 5: Run local Worker tests** and confirm PASS.
- [ ] **Step 6: Commit** with `feat: add Cogmax Cloudflare runtime adapter`.

### Task 6: Asynchronous extraction and consolidation

**Files:**
- Create: `crates/cogmax-pipeline/src/lib.rs`
- Create: `crates/cogmax-pipeline/src/model_provider.rs`
- Create: `crates/cogmax-pipeline/src/worker.rs`
- Create: `workers/cogmax-worker/src/queue_consumer.rs`
- Test: `crates/cogmax-pipeline/tests/pipeline.rs`

**Interfaces:**
- Consumes events/candidates from storage and a `ModelProvider` trait.
- Produces idempotent extraction, consolidation and retry results suitable for Cloudflare Queues.

- [ ] **Step 1: Write failing pipeline tests** for structured extraction, malformed model output, duplicate delivery, retry and dead-letter classification.
- [ ] **Step 2: Run pipeline tests** and confirm failure.
- [ ] **Step 3: Define the provider-neutral `ModelProvider` interface** and validated candidate schema; no model vendor is hard-coded.
- [ ] **Step 4: Implement extraction and consolidation worker logic** using the event id as idempotency key.
- [ ] **Step 5: Implement Queue consumer retry/ack behavior** and persist terminal failures.
- [ ] **Step 6: Run pipeline tests** and confirm PASS.
- [ ] **Step 7: Commit** with `feat: add Cogmax asynchronous memory pipeline`.

### Task 7: R2, Parquet, snapshots and Git synchronization

**Files:**
- Create: `crates/cogmax-export/src/lib.rs`
- Create: `crates/cogmax-export/src/parquet.rs`
- Create: `crates/cogmax-export/src/markdown.rs`
- Create: `crates/cogmax-export/src/git_sync.rs`
- Create: `workers/cogmax-worker/src/sync_worker.rs`
- Test: `crates/cogmax-export/tests/snapshot.rs`

**Interfaces:**
- Consumes versioned memory state and audit events from Tasks 2–3.
- Produces deterministic snapshot manifests, Markdown/Parquet artifacts and asynchronous Git sync status.

- [ ] **Step 1: Write failing snapshot tests** for deterministic output, manifest hashes, restore round-trip and Git outage behavior.
- [ ] **Step 2: Run snapshot tests** and confirm failure.
- [ ] **Step 3: Implement deterministic Markdown and Parquet export** with no secrets or raw sensitive fields in default output.
- [ ] **Step 4: Implement snapshot manifest and restore validation** using content hashes and schema version.
- [ ] **Step 5: Implement Git provider synchronization** as grouped asynchronous commits with optimistic ref updates and retryable failures.
- [ ] **Step 6: Implement R2 object persistence** for snapshots and raw event archives.
- [ ] **Step 7: Run snapshot tests** and confirm PASS, including Git outage recovery.
- [ ] **Step 8: Commit** with `feat: add Cogmax snapshots and Git sync`.

### Task 8: End-to-end verification and release packaging

**Files:**
- Create: `tests/e2e/skill-first-flow.rs`
- Create: `Dockerfile`
- Create: `README.md`
- Modify: `skill/SKILL.md`
- Test: `tests/e2e/skill-first-flow.rs`

- [ ] **Step 1: Write the end-to-end scenario**: activate skill, recall, learn explicit correction, consolidate, start a new session and verify recall.
- [ ] **Step 2: Run the E2E test** and confirm the complete path fails before wiring is complete.
- [ ] **Step 3: Wire local runtime, skill package and test fixtures** without exposing internal storage details to the skill user.
- [ ] **Step 4: Run unit, integration and E2E tests** with `cargo test --workspace`.
- [ ] **Step 5: Run formatting, linting and dependency audit** with `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings` and the selected audit tool.
- [ ] **Step 6: Build the local release artifact and verify the Docker image** starts, serves health and executes the E2E flow.
- [ ] **Step 7: Commit** with `chore: package Cogmax MVP`.

## Execution Notes

- A ordem de execução é offline-first: Tasks 1–4 e um runtime local persistente devem estar verdes antes da integração Cloudflare.
- Cloudflare deployment verification is separate from local tests and must report Worker, Durable Object, Queue and R2 evidence independently.
- Do not claim a live Git provider, Cloudflare deployment or model-provider integration from local tests alone.
- If the Wasm build cannot share the native Rust core without unsafe runtime assumptions, keep the domain crate shared and provide separate storage/runtime adapters.
