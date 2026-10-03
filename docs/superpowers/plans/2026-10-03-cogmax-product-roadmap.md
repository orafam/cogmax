# Cogmax Product Roadmap

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans or superpowers:subagent-driven-development to implement this roadmap task-by-task. Each slice must be independently tested before moving forward.

**Goal:** Transform the working offline-first Cogmax MVP into a distributable, secure and agent-agnostic memory product while preserving the skill-first experience.

**Architecture:** Keep the local Rust service and SQLite as the reference implementation. The Skill remains the only user-facing integration surface; API, MCP, storage and synchronization are implementation details. Add remote synchronization only after local behavior, identity and data contracts are stable.

**Tech Stack:** Rust, Axum, SQLite, Parquet, Bash Skill, Next.js, GitHub Actions, GitHub Releases; later Cloudflare Workers and Durable Objects.

**Spec:** `docs/superpowers/specs/2026-10-02-rrc-skill-first-memory-design.md`

## Global Constraints

- Skill-first: the user activates a Skill and does not configure MCP, API, database or infrastructure.
- Offline-first: all core memory operations must work without network access.
- Project-aware: memories must retain user and project scope.
- Decisions must preserve the decision and the known rationale; the system must not invent a reason.
- Explicit user instructions outrank recalled memory.
- Sensitive content must not be imported or exposed.
- Every release must include platform binaries, `SHA256SUMS` and the matching Skill package.
- Preserve the local SQLite implementation as the source of truth until remote synchronization is proven.

## Review Focus

- A Skill installed outside the repository must find the installed binary and work from a clean directory.
- A user must not see another user's project memory after authentication or scope changes.
- Re-importing the same source must be idempotent and must not erase explicit memories.
- Conflicting decisions must be represented as conflict/supersession, never silently merged.
- A failed cloud sync must preserve the local committed memory and provide a retryable result.

## Roadmap

### Phase 1 — Distribution foundation

**Outcome:** A new user can download Cogmax, activate the Skill and use memory without knowing the implementation details.

### Task 1: Skill installer contract

**Files:**
- Modify: `crates/cogmax-cli/src/install.rs`
- Modify: `skill/scripts/memory`
- Test: `crates/cogmax-cli/src/install.rs` unit tests and an external-directory smoke test

**Deliverable:** `cogmax install` installs the binary, `SKILL.md` and the executable Skill script; the script resolves `cogmax` from `PATH` or `COGMAX_BIN` and never assumes a checkout.

**Acceptance:** install, start, learn and recall succeed from `/tmp` with no repository-relative paths.

### Task 2: Release and home consistency

**Files:**
- Modify: `.github/workflows/release.yml`
- Modify: `api/app/page.tsx`
- Modify: `api/app/styles.css`

**Deliverable:** every tagged release automatically includes the Skill package, and the home explains the exact user flow: download, add to the agent, activate; no manual MCP/API/database setup.

**Acceptance:** tagged CI passes on Linux, macOS Intel, macOS ARM64 and Windows; checksums validate; the home download URL points to the current release.

### Phase 2 — Agent and project onboarding

### Task 3: Agent-specific discovery adapters

**Files:**
- Modify: `crates/cogmax-discovery/src/lib.rs`
- Create: `crates/cogmax-discovery/src/sources/`
- Test: fixture-based Codex, Claude, Kimi and generic source tests

**Deliverable:** discovery reports detected agents, source paths, file counts and project scopes without scanning unrelated home-directory data.

**Acceptance:** each adapter imports supported Markdown/JSON fixtures, rejects sensitive files, preserves source provenance and remains deterministic.

### Task 4: Guided onboarding command

**Files:**
- Modify: `crates/cogmax-cli/src/main.rs`
- Create: `crates/cogmax-cli/src/onboarding.rs`
- Modify: `skill/SKILL.md`
- Test: CLI integration tests

**Deliverable:** `cogmax discover` produces a user-readable onboarding summary and `cogmax import --preview` explains what will be imported, grouped by agent and project.

**Acceptance:** preview never writes memory; apply is explicit; rebuild reports what it removes and preserves.

### Phase 3 — Memory quality

### Task 5: Structured decision and project extraction

**Files:**
- Modify: `crates/cogmax-discovery/src/lib.rs`
- Modify: `crates/cogmax-domain/src/memory.rs`
- Test: decisions with/without rationale, projects, procedures, corrections and ambiguous notes

**Deliverable:** imported memories carry reliable kind, confidence, authority, source and project metadata; missing rationale is explicit.

**Acceptance:** no extractor invents causality; low-confidence candidates do not become active memories.

### Task 6: Consolidation and conflict model

**Files:**
- Modify: `crates/cogmax-domain/src/memory.rs`
- Modify: `crates/cogmax-storage/src/lib.rs`
- Modify: `crates/cogmax-core/src/lib.rs`
- Test: duplicate, supersession, revocation and conflicting decision scenarios

**Deliverable:** repeated observations consolidate safely; newer explicit corrections supersede older memories; conflicts remain visible until resolved.

**Acceptance:** no silent overwrite of explicit decisions and all transitions are auditable locally.

### Task 7: Hybrid recall and explainable ranking

**Files:**
- Modify: `crates/cogmax-core/src/lib.rs`
- Modify: `crates/cogmax-api/src/lib.rs`
- Test: lexical ranking, type/project filters, exact scope isolation and empty results

**Deliverable:** recall combines lexical match, project scope, kind, confidence, authority and recency; response metadata explains why a memory was selected.

**Acceptance:** broad queries do not bury decisions and project-filtered queries never cross scope boundaries.

### Phase 4 — Security and operational readiness

### Task 8: Identity and authorization boundary

**Files:**
- Modify: `crates/cogmax-api/src/lib.rs`
- Modify: `crates/cogmax-storage/src/lib.rs`
- Create: `docs/security.md`
- Test: authenticated/unauthenticated requests, tenant isolation, invalid scopes and rate limits

**Deliverable:** API and MCP calls require a platform-issued identity/token outside local development, with explicit user/project authorization.

**Acceptance:** unauthenticated calls fail closed; cross-user and cross-project reads are rejected; secrets never enter logs or recall output.

### Task 9: Local backup, restore and export contract

**Files:**
- Modify: `crates/cogmax-export/src/lib.rs`
- Modify: `crates/cogmax-cli/src/main.rs`
- Test: deterministic Markdown/Parquet snapshots, manifest hashes, restore and corrupted snapshot rejection

**Deliverable:** users can create, inspect and restore versioned local snapshots without losing explicit memories.

**Acceptance:** restored data produces the same IDs, scopes and recall results as the source snapshot.

### Phase 5 — Synchronization and remote runtime

### Task 10: Git synchronization adapter

**Files:**
- Modify: `crates/cogmax-export/src/lib.rs`
- Create: `crates/cogmax-export/src/git.rs`
- Test: clean sync, changed snapshot, conflict and offline retry

**Deliverable:** Git is an optional backup/sync destination, never a prerequisite for local use.

**Acceptance:** sync is idempotent, produces reviewable commits and never pushes secrets or unresolved conflicts automatically.

### Task 11: Cloudflare adapter

**Files:**
- Create: `cloudflare/worker/`
- Create: `cloudflare/durable-object/`
- Modify: `docs/architecture.md`
- Test: contract tests against the local API and remote integration tests

**Deliverable:** a remote runtime provides authenticated synchronization while preserving the same memory contract as local Cogmax.

**Acceptance:** offline writes reconcile deterministically, duplicate events remain idempotent and remote failures do not block local recall.

### Phase 6 — Product hardening

### Task 12: Observability, documentation and compatibility

**Files:**
- Modify: `docs/`
- Modify: `.github/workflows/`
- Create: `CHANGELOG.md`
- Test: release smoke matrix and upgrade test

**Deliverable:** installation, upgrade, rollback, privacy, data deletion and troubleshooting are documented; CI covers release assets and Skill compatibility.

**Acceptance:** a clean machine can install the latest release, activate the Skill, import memories, recall a project decision and upgrade without data loss.

## Recommended execution order

1. Complete Tasks 1–2 and verify the distribution loop.
2. Complete Tasks 3–4 to make onboarding agent-agnostic.
3. Complete Tasks 5–7 before adding any semantic or remote dependency.
4. Complete Tasks 8–9 before exposing a public multi-user endpoint.
5. Complete Tasks 10–11 only after local snapshots and conflict behavior are stable.
6. Finish with Task 12 and a clean-machine acceptance test.

## Definition of Done

Cogmax is ready for general use when a clean machine can download one release, activate one Skill, run offline, import multiple agent memories with project scopes, recall decisions with rationale, preserve data across upgrade, and optionally synchronize through an authenticated remote runtime.
