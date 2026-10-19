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
