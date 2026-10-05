# Slice 1: Autonomous Skill Loop

**Goal:** Make the installed Skill orchestrate the runtime internally.

**Files:**
- Modify: `skill/SKILL.md`
- Modify: `skill/scripts/memory`
- Modify: `crates/cogmax-cli/src/main.rs`
- Test: `crates/cogmax-cli/tests/offline_runtime.rs`

**Interfaces:**
- `skill/scripts/memory` remains the only executable called by the agent.
- Internal actions use `cogmax discover`, `cogmax import --preview`, and `cogmax import --apply`; these names must not appear in user-facing responses.

- [ ] Add a first-run marker under the Cogmax data directory and make discovery/import run once per installation.
- [ ] Add an explicit internal action that returns machine-readable summary data while keeping conversational output natural.
- [ ] Make the Skill say: inspect context → recall → if first run, summarize discovered sources → ask confirmation → import after confirmation.
- [ ] Add an offline integration test from `/tmp` that installs the Skill, runs first activation twice, and proves the second activation is idempotent.
- [ ] Run `cargo test -p cogmax-cli --test offline_runtime` and commit `feat: make skill onboarding autonomous`.
