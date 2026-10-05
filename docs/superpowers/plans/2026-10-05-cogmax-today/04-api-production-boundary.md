# Slice 4: Production API Boundary

**Goal:** Make the HTTP surface fail closed and safe for a real authenticated caller.

**Files:**
- Modify: `crates/cogmax-api/src/lib.rs`
- Modify: `docs/security.md`
- Test: `crates/cogmax-api/src/lib.rs`

**Interfaces:**
- `COGMAX_API_TOKEN` protects `/learn` and `/recall`.
- `COGMAX_API_USER` constrains scopes to `user:<identity>` and its projects.
- `/health` remains public; invalid input returns `4xx`, never a panic.

- [ ] Replace `expect` in request handlers with structured `400/409/500` responses.
- [ ] Add authenticated supersede/revoke endpoint with user/project authorization.
- [ ] Add bounded request body size and rate-limit tests.
- [ ] Ensure inferred authority cannot be forged through the public endpoint.
- [ ] Run API tests, workspace tests, clippy, and commit `feat: harden authenticated api boundary`.
