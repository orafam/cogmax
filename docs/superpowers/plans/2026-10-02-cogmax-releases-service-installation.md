# Cogmax Releases and Service Installation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Publish cross-platform Cogmax binaries through GitHub Releases and teach the project skill to install and manage the local service on Windows, Linux, and macOS.

**Architecture:** The release workflow builds the existing `cogmax-cli` binary for five targets, packages assets, generates checksums, and publishes a release for `v*` tags. The CLI owns platform-native service files and lifecycle commands; the skill downloads the matching release asset, verifies it, installs the binary, and delegates lifecycle operations to the CLI.

**Tech Stack:** Rust/Cargo, Axum, GitHub Actions, systemd, launchd, Windows Service/PowerShell, Markdown skill instructions.

**Spec:** In-chat approved design: public GitHub Release assets plus native CLI installation, without graphical installers in the first version.

## Global Constraints

- Preserve the existing memory API and SQLite behavior.
- Support Linux x86_64, macOS x86_64, macOS arm64, and Windows x86_64.
- Use native service managers: systemd, launchd, and Windows Service.
- Verify downloaded release assets with SHA-256 before installation.
- Do not require kubectl, a cluster, or a remote database.

## Review Focus

- Unsupported operating system or architecture produces a clear error and no partial installation.
- Service paths containing spaces are safely quoted in generated configurations.
- Reinstall/update does not destroy the existing `.cogmax` data directory.
- Missing or invalid checksum prevents installation.
- `import` and existing API behavior remain unchanged.

### Task 1: Platform installation primitives

**Files:**
- Modify: `crates/cogmax-cli/src/main.rs`
- Create: `crates/cogmax-cli/src/install.rs`
- Test: `crates/cogmax-cli/src/install.rs` unit tests

- [ ] Add platform-neutral artifact selection, install paths, checksum validation, and service configuration generation.
- [ ] Add lifecycle commands: `install`, `uninstall`, `start`, `stop`, `restart`, `status`.
- [ ] Add tests for target selection, checksum verification, and platform configuration escaping.
- [ ] Run focused CLI tests, then `cargo test --workspace`.

### Task 2: GitHub Actions release pipeline

**Files:**
- Create: `.github/workflows/release.yml`

- [ ] Build the CLI for the five approved targets on `v*` tags.
- [ ] Package binaries with stable names and generate `SHA256SUMS`.
- [ ] Publish all assets to a GitHub Release using the tag version.
- [ ] Validate YAML shape and document the tag-triggered release contract.

### Task 3: Skill installation and operations guide

**Files:**
- Modify: `skill/SKILL.md`

- [ ] Document release asset resolution, checksum verification, install paths, and native service lifecycle.
- [ ] Document platform detection and safe handling of unsupported platforms.
- [ ] Document update, status, logs, uninstall, and data-preservation behavior.
- [ ] Keep the existing memory behavior and privacy constraints intact.

### Task 4: Verification

- [ ] Run formatting and the complete Rust test suite.
- [ ] Build the release binary locally.
- [ ] Inspect the final diff and report local verification versus remote GitHub Actions verification.
