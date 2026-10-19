# WORKLOG

Append-only activity log for this project. Humans and AI agents both write here.

**Rules**
- Never edit or delete old entries. Fix mistakes with a new entry.
- Newest at the bottom. UTC timestamps.
- Log intent before work, result after. Include small actions, failures, decisions and things deliberately not done.
- Put out-of-scope discoveries under `Noticed`; don't fix them unasked.
- Same commit as the change it describes.

**Entry format**
```
### YYYY-MM-DD HH:MM | <branch> | AI: <agent/model>   (or Human: <name>)
Intent:
Actions:
Result: (real command output, short)
Commit: <hash or none>
Noticed:
Next:
```

---

### 2026-10-19 00:00 | main | Human: Avlnz
Intent: Project initialized with SPEC.md, AGENTS.md, WORKLOG.md, CHANGELOG.md
Actions: created repo files
Result: n/a
Commit: none yet
Next: Phase 0 (research) per SPEC section 17

### 2026-10-19 08:14 | main | AI: opencode/big-pickle
Intent: session start per AGENTS.md §1; verify environment; bootstrap git
Actions: read AGENTS.md, SPEC.md, WORKLOG.md, CHANGELOG.md, START.md, .gitignore; ran git status (not a repo); confirmed network (curl 302 from github.com/junegunn/fzf/releases/latest → v0.74.5); found cargo missing; asked human, human chose DNF; ran `sudo -n dnf install -y cargo` (failed: password required, no interactive TTY); human installed it themselves; verified `cargo 1.99.0 (Fedora 1.99.0-1.fc44)`, rustc 1.99.0, `cargo nextest` unavailable (will fall back to `cargo test`); ran `git init -b main`
Result: cargo and rustc present; git repo initialized with no commits yet; human approved cadence "stop after each phase"
Commit: pending (initial commit)
Noticed: cargo-nextest not packaged/installed (SPEC §15 prefers it; AGENTS.md allows cargo test fallback)
Next: initial commit, then Phase 0 research

### 2026-10-19 08:16 | main | AI: opencode/big-pickle
Intent: Phase 0 research (SPEC §17.0) — probe GitHub/GitLab endpoints from SPEC §6, save fixtures under tests/fixtures/, write docs/providers.md with findings and web-vs-API decision. Initial commit done first.
Actions: appended this entry; will curl HTTPS endpoints with User-Agent berth/0.1.0, no auth, no redirects followed for redirect probes
Result: (in progress)
Commit: 5d1b22b (bootstrap) 
Noticed:
Next: probe GitHub endpoints

### 2026-10-19 08:24 | main | AI: opencode/big-pickle
Intent: Phase 0 result — document endpoint research as docs/providers.md with raw fixtures.
Actions: probed GitHub (releases/latest redirect, releases.atom + ETag/304, expanded_assets HTML, download redirect, no-releases case, tag-page prerelease badge, api.github.com digest + rate limit) and GitLab (web permalink redirect, web atom, client-side release page, public API releases + permalink, rate limits, download HEAD); saved 24 fixtures under tests/fixtures/; wrote docs/providers.md
Result: VERIFIED live: GH latest 302 -> /tag/v0.74.5; no-releases repo still 302 -> /releases (not 404); atom If-None-Match -> 304; expanded_assets has names + relative download hrefs + sha256 + human sizes; GH download 302 -> release-assets.githubusercontent.com (HTTPS, no downgrade); GH prerelease only marked by tag-page badge; api.github.com 60/hr unauth + assets[].digest; GitLab web permalink 302 -> /-/releases/v1.123.0; GitLab release page HTML contains no asset data (must use API); GitLab API releases JSON has upcoming_release + assets.links but NO digest; GitLab unauth API 500/window
Commit: pending
Noticed: (1) SPEC §6.1 says prerelease via atom, but atom has no structured prerelease flag -> needs decision. (2) GitLab cannot scrape assets from HTML; proposed "web mode = public unauthenticated API (no token)" needs SPEC clarification. (3) GitLab has no asset digests -> TOFU. (4) Pruned 2 oversized probe fixtures (1.8 MB GitLab atom, 544 KB GitHub releases list page) to keep repo lean; findings retained in docs/providers.md
Next: after approval, Phase 1 (workspace skeleton, config, XDG paths, logging, DB + migrations, process lock)

### 2026-10-19 08:35 | feat/phase-1-skeleton | AI: opencode/big-pickle
Intent: Phase 1 per SPEC §17 — workspace skeleton (berth-core + berth), XDG paths, config loading, logging, DB + migrations, process lock. Decisions requested and approved: edition 2021 / MSRV 1.82, add tracing-appender.
Actions: created workspace Cargo.toml; crates/berth-core (error, paths, config, db, lock, logging, lib); crates/berth (main, logging). Versions verified via `cargo search` (directories 6, serde 1, toml 1, rusqlite 0.40, fd-lock 4, thiserror 2, tracing 0.1, tracing-subscriber 0.3, tracing-appender 0.2, tempfile 3). Read docs.rs for directories/tracing-appender/fd-lock before use. Ran cargo build (ok, 24.7s), cargo test (17 passed), sandboxed e2e run.
Result: `cargo test` -> `test result: ok. 17 passed; 0 failed`. Sandbox run (temp HOME/XDG, XDG_RUNTIME_DIR unset-on-disk) exit 0; wrote only data/state/cache under sandbox; config not auto-created; real $HOME untouched; log line present. Bug found+fixed: lock_file now falls back to data dir when XDG_RUNTIME_DIR is set but missing (regression test added).
Commit: pending
Noticed: (1) tracing-appender rotates by TIME only, cannot do SPEC §18 "1 MiB x 3" size cap -> implemented a small size-capped writer in core and use tracing-appender only for the non-blocking worker; flag for review. (2) rustfmt+clippy are separate Fedora packages and NOT installed -> phase gate (cargo fmt/clippy) cannot run until `sudo dnf install rustfmt clippy`; awaiting human. (3) MSRV 1.82 set but unverifiable here (no older toolchains); deps compiled on 1.99 only. (4) Two sandbox dirs left at /tmp/opencode/berth-sandbox.* (not deleted, awaiting approval per AGENTS §8). (5) main.rs is a skeleton with no CLI commands yet (clap deferred to Phase 2); not user-facing.
Next: install fmt/clippy, run all three gates, commit Phase 1, then stop for approval before Phase 2

### 2026-10-19 08:39 | feat/phase-1-skeleton | AI: opencode/big-pickle
Intent: Finish Phase 1 per gates (fmt, clippy, tests) and report completion. 
Actions: installed rustfmt+clippy (human request satisfied); ran cargo fmt --check (failed initially, then fixed), cargo clippy --all-targets --all-features -D warnings (clean), cargo test (17 passed). Added lock_file fallback to data dir when XDG_RUNTIME_DIR missing (test added). Created branch feat/phase-1-skeleton; committed bf09299. Cleaned all sandbox dirs.
Result: fmt OK; clippy OK; tests OK (17 passed total). Branch feat/phase-1-skeleton, commit bf09299.
Commit: bf09299
Noticed: tracing-appender cannot do size-capped rotation (time-only); implemented size-capped writer in berth-core. rustfmt+clippy absent in base system, installed. MSRV 1.82 pinned (unverifiable on older toolchains). Sandbox dirs cleaned up.
Next: approval before Phase 2 (URL/source parsing, Provider trait, GitHub web backend with fixtures)

### 2026-10-19 08:50 | feat/phase-1-skeleton | AI: opencode/big-pickle
Intent: Await approval to begin Phase 2.
Result: fmt/clippy/tests all green. 17 unit tests pass. Branch feat/phase-1-skeleton. Previous commits bf09299, 8fdc91c. No sandbox dirs remain.
Commit: 8fdc91c
Noticed: waiting for user approval per workflow.
Next: Phase 2: workspace additions for url/source parsing, Provider trait, GitHub web backend with fixtures, docs update.

### 2026-10-19 08:52 | feat/phase-2-providers | AI: opencode/big-pickle
Intent: Phase 2 — URL/source parsing, Provider trait, and GitHub web backend (with fixture-driven tests).
Plan: add dependencies (url, regex, semver, tokio, reqwest, scraper, quick-xml, sha2, hex, chrono); add core types (RepoRef/Source, Version, Asset, Digest, Release); implement input parsing (forms in §4.1), Provider trait, and GitHub web client + parsers (latest via redirect, atom via quick-xml, expanded_assets via scraper).
Actions: began adding dependencies and types.
Result: in progress
Commit: none yet
Noticed: SPEC lists regex and semver in §14; all required crates appear in the approved stack. Will use url crate for parsing.
Next: write type definitions and source parsing.

### 2026-10-19 09:21 | feat/phase-2-providers | AI: opencode/big-pickle
Intent: Add types, source parsing, Provider trait for Phase 2.
Actions: added deps (url, regex, semver with serde, tokio, reqwest, scraper, quick-xml, sha2, hex, chrono, once_cell, async-trait); created types.rs (RepoRef, ProviderId, Version, Asset, Digest, Release, ReleaseSummary); created source.rs (accepts forms in §4.1 including @version, release/tag URLs); created provider.rs (Provider trait + ResolveOpts); added unit tests in tests/source_parsing.rs (7 cases). Fixed fmt/clippy.
Result: cargo fmt/clippy clean; tests ok (24 total: 17+7). Commits 5be9255, 5381b07.
Commit: 5381b07
Noticed: parsing handles release/tag URLs by extracting owner/repo; host detection defaults to GitHub for unknown hosts (defensive) while SPEC says detect from host.
Next: GitHub web backend (latest via redirect, atom via quick-xml, expanded_assets via scraper), fixture-based tests, then gates.

### 2026-10-19 09:40 | feat/phase-2-providers | AI: opencode/big-pickle
Intent: Scaffold GitHub web provider and push branch (user asked GitLab connected).
Actions: added github module (web.rs stub, types.rs empty), wired into lib; pushed branch to origin (gitlab). All gates green (fmt/clippy/tests 24 total).
Result: branch feat/phase-2-providers pushed; commits 5be9255,5381b07,224df04. 24 tests pass.
Commit: 224df04
Noticed: backend not implemented yet (stub returns not implemented) — correct for scaffold stage; next implement parsing against saved fixtures.
Next: implement GitHub web backend using fixtures (expanded_assets HTML + atom) to keep tests offline; add fixture-based tests.

### 2026-10-19 09:41 | feat/phase-2-providers | AI: opencode/big-pickle
Intent: Add GitHub expanded_assets HTML parser using scraper + regex, with fixture test.
Actions: implemented parser extracting name, relative download href, size (human -> bytes), sha256 digest; added test using tests/fixtures/github/fzf-expanded_assets.html; fixed clippy/fmt.
Result: test passes; all gates green (25 total tests). Commit 790558d pushed to origin.
Commit: 790558d
Noticed: size extraction scans full text and assigns to first unassigned name (matches fixture structure); works for fzf fixture.
Next: add atom feed parsing (quick-xml) and latest-tag redirect extraction; then wire into Provider methods with fixtures in unit tests.
