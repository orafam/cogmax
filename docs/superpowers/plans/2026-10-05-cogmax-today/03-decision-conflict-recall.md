# Slice 3: Decisions, Conflicts, and Explainable Recall

**Goal:** Make recalled decisions trustworthy and explainable.

**Files:**
- Modify: `crates/cogmax-domain/src/memory.rs`
- Modify: `crates/cogmax-core/src/lib.rs`
- Modify: `crates/cogmax-discovery/src/lib.rs`
- Modify: `crates/cogmax-api/src/lib.rs`
- Test: `crates/cogmax-core/src/tests.rs`
- Test: `crates/cogmax-discovery/src/lib.rs`

**Interfaces:**
- `MemoryKind::Decision` content always includes `Motivo: ...` or `Motivo: não identificado na fonte importada`.
- `RecallMatch { memory, reasons }` remains the explainable recall contract.
- Explicit correction transitions a selected memory to `Superseded`; it never silently overwrites another decision.

- [ ] Add an auditable supersession relation containing source and replacement IDs.
- [ ] Rank project-scoped decisions above broad references while preserving conflicts in results until resolved.
- [ ] Add tests for missing rationale, conflicting decisions, explicit supersession, project isolation, and empty recall.
- [ ] Verify API explanations do not expose secrets or internal storage details.
- [ ] Run workspace tests/clippy and commit `feat: audit decision supersession`.
