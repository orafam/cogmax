# Slice 6: Clean-Machine Acceptance and Release Gate

**Goal:** Prove the user journey on a clean environment and make CI enforce it.

**Files:**
- Modify: `.github/workflows/ci.yml` or create the missing workflow
- Modify: `.github/workflows/release.yml`
- Modify: `skill/SKILL.md`
- Modify: `cloudflare/worker/package.json`
- Create: `cloudflare/worker/package-lock.json` if absent in the target branch
- Test: `crates/cogmax-cli/tests/offline_runtime.rs`

**Interfaces:**
- Rust gate: `cargo test --workspace` and `cargo clippy --workspace --all-targets -- -D warnings`.
- Worker gate: `npm ci && npm run typecheck` in `cloudflare/worker`.
- Release gate: platform archives, `SHA256SUMS`, and matching Skill archive.

- [ ] Add CI jobs for Rust, Skill external-directory smoke test, Worker typecheck, and snapshot round trip.
- [ ] Add upgrade test proving explicit memories survive reinstall and rebuild.
- [ ] Verify release asset URLs and Skill version match automatically.
- [ ] Run the full clean-machine matrix and record the exact commands/results in `docs/compatibility.md`.
- [ ] Commit `chore: add clean machine release gate` and tag only after CI is green.
