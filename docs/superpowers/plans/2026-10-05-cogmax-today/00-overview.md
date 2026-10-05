# Cogmax Today Completion Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans. Each slice is a separate file and must be implemented, tested, and committed independently.

**Goal:** Finish the Skill-first product loop today so a user installs one Skill, activates it in an agent, and uses memory without knowing the CLI, API, database, Git, or Cloudflare.

**Architecture:** The installed Skill is the only user-facing surface. It delegates to the local Rust runtime, which remains offline-first and SQLite-backed. Remote Git and Cloudflare synchronization are optional post-commit adapters and never block local recall.

**Tech Stack:** Rust, SQLite, Bash Skill, TypeScript Cloudflare Worker, Durable Objects, GitHub Actions.

**Spec:** `docs/superpowers/specs/2026-10-02-rrc-skill-first-memory-design.md`

## Execution order

1. `01-skill-autonomous-loop.md`
2. `02-import-confirmation.md`
3. `03-decision-conflict-recall.md`
4. `04-api-production-boundary.md`
5. `05-snapshot-sync-retry.md`
6. `06-clean-machine-acceptance.md`

Each slice ends with its own commit. If time runs short, stop after Slice 2: that is the minimum product-complete user experience.

## Definition of done for today

- User-facing docs never ask the user to run CLI commands.
- A clean directory can activate the Skill and use `memory recall`/`memory learn`.
- First activation discovers existing agent memories and requests conversational confirmation before batch import.
- Decisions preserve known rationale and conflicts remain explicit.
- Local backup can be inspected, restored, retried, and synced without accidental overwrite.
- CI validates Rust, Worker typecheck, Skill smoke test, and release assets.
