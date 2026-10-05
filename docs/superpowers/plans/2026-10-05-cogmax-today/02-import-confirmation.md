# Slice 2: Conversational Import Confirmation

**Goal:** Ensure batch import never occurs silently and the user never needs CLI knowledge.

**Files:**
- Create: `crates/cogmax-cli/src/onboarding.rs` additions only if needed
- Modify: `skill/SKILL.md`
- Modify: `skill/scripts/memory`
- Test: `crates/cogmax-cli/tests/offline_runtime.rs`

**Interfaces:**
- Preview result must expose total files, agents, projects, kinds, skipped low-confidence count, and sensitive-file count.
- Apply must require an internal confirmation flag/token created by the Skill, not a human CLI argument.

- [ ] Extend the preview summary with sensitive/skipped counts and a stable confirmation digest.
- [ ] Require the same digest for apply; reject stale or modified previews.
- [ ] Make the Skill ask one conversational confirmation containing only the summary and never a command.
- [ ] Test reject-without-confirmation, accept-with-confirmation, stale-preview rejection, and explicit memories preserved by rebuild.
- [ ] Run workspace tests and commit `feat: require conversational import confirmation`.
