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
