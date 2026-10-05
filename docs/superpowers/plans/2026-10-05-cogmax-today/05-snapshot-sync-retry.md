# Slice 5: Snapshot and Sync Retry

**Goal:** Make backup and synchronization safe to retry after offline or remote failure.

**Files:**
- Modify: `crates/cogmax-export/src/lib.rs`
- Modify: `crates/cogmax-export/src/git.rs`
- Modify: `crates/cogmax-cli/src/main.rs`
- Test: `crates/cogmax-export/src/tests.rs`

**Interfaces:**
- `verify_manifest(markdown, manifest)` rejects altered content.
- `sync_snapshot(repo, snapshot, destination, message)` is idempotent, never pushes, and returns `SnapshotConflict` for a different destination snapshot.

- [ ] Add retry metadata containing snapshot hash, last attempt, and failure reason without storing secrets.
- [ ] Make restore transactional: validate all files and counts before the first database write.
- [ ] Add tests for identical retry, changed snapshot conflict, corrupted JSON, and partial-write prevention.
- [ ] Add a natural-language Skill instruction for “backup failed, retry later”.
- [ ] Run export tests, CLI smoke test, workspace tests, and commit `feat: make snapshot retry transactional`.
