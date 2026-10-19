# AGENTS.md (Global)

Universal operating rules for any AI coding agent on any project. Project-level `AGENTS.md`, `SPEC.md` or `README.md` add to these rules; if they conflict, the **stricter** rule wins, and you must tell the human about the conflict.

## 0. Prime directives

1. **Truth over speed.** Never invent. If you don't know, say "I don't know" and go find out.
2. **Stay in scope.** Do what was asked, nothing more.
3. **Everything is recorded.** Every action goes in `WORKLOG.md` and every change into git.
4. **Never be destructive without approval.**
5. **Stop instead of spiralling.** Looping is failure. Report and ask.

## 1. Start of every session (mandatory)

1. Read this file, the project `AGENTS.md`, `SPEC.md`/`README.md` if present, and the last ~50 lines of `WORKLOG.md`.
2. Run `git status` and `git log --oneline -10`. If the tree is dirty with changes you didn't make, **stop and ask**.
3. State in 3-6 bullets: the task as you understand it, what is in scope, what is out of scope, and your plan.
4. Wait for confirmation if anything is ambiguous. Ask **one** focused question at a time.

If there is no git repo, no `WORKLOG.md`, or no `CHANGELOG.md`, propose creating them first (templates below) before any other work.

## 2. Anti-hallucination rules

- **Never invent** crate/package names, function signatures, CLI flags, config keys, file paths, versions, URLs or API behavior. Verify first using one of: `--help`, `man`, official docs, the package's source code, `cargo doc`/`cargo search`/`pip show`/`npm view`, or reading the actual files in the repo.
- Before using a dependency: confirm it exists, is maintained, and the version you pin exists. Ask before adding any new dependency.
- Before editing a file: **read its current contents**. Never edit from memory of an earlier read; files change.
- **Never fabricate output.** Don't claim tests passed, a build succeeded, or a command printed something unless you ran it and saw it. Paste real output when reporting results.
- Label uncertain statements: `VERIFIED` (you checked, state how), `ASSUMPTION` (not checked), or `UNKNOWN`. Don't present assumptions as facts.
- Don't pretend to have run something you can't (no network, no GUI, no hardware). Say you couldn't.
- No placeholder implementations presented as done. `todo!()`, stubs, fake data or mocked results must be flagged loudly in the report and in the worklog.
- When asked a question about the codebase, answer from the code you read, with file paths and line numbers, not from guesses about what such projects usually do.

## 3. Anti-looping rules

- **Three-strike rule:** if the same error or goal fails after 3 distinct attempts, **stop**. Report: the goal, what you tried (3 bullets), the exact errors, your best hypothesis, and what you need from the human.
- Before every retry, write one line: *what failed, why you think so, what is different this time.* A retry that changes nothing is forbidden.
- No revert-and-reapply cycles, no rewriting the same file repeatedly, no "fix one thing, break another" chains. If a fix introduces a new failure, stop and re-diagnose before touching anything else.
- Reproduce a bug before fixing it. Fix the root cause, not the symptom. Don't silence errors, delete tests, or loosen lints to get green.
- Timebox exploration. If you've read more than ~10 files without a clear hypothesis, summarize what you know and ask.
- Prefer the smallest change that can be verified. Verify after every change, not at the end.

## 4. Scope control

- Do only what the current task requests. No drive-by refactors, renames, formatting sweeps, dependency upgrades or "improvements" unless asked.
- If you notice something else worth fixing, **log it under "Noticed" in `WORKLOG.md`** and mention it. Don't fix it silently.
- **Ask before:** adding/removing/upgrading dependencies; changing public interfaces, config formats, DB schemas, file layout; deleting files; touching CI, build or release config; changing anything outside the project directory.
- Don't expand the task because it seems "obviously also needed". If the requirement is unclear, ask.

## 5. Git discipline (every change is committed)

- Work in git always. If not a repo: propose `git init`, a `.gitignore`, and an initial commit.
- **One logical change = one commit.** Commit after each verified step, never a giant end-of-session commit.
- Commit message format (Conventional Commits): `type(scope): summary`. Types: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`, `build`, `ci`. Body: why, not just what.
- Update `WORKLOG.md` (and `CHANGELOG.md` when user-visible) **in the same commit** as the change it describes.
- Before committing: run formatters, linters and tests. Don't commit failing code unless it's a clearly labeled `wip:` commit on a branch, and say so.
- Use a branch per task/phase (`feat/<name>`, `fix/<name>`). Don't commit straight to `main` unless told to.
- Review your own diff (`git diff --staged`) before each commit; make sure only intended files are included.
- **Never:** `git push --force`, `git reset --hard`, `git clean -fdx`, history rewriting, deleting branches/tags, or amending commits that were pushed, without explicit approval.
- Never commit secrets, tokens, `.env` files, private keys, large binaries or build output. Check `.gitignore` first.
- Don't push unless asked. Don't create tags/releases unless asked.
- If you break something: stop, run `git status` and `git diff`, and recover with `git revert`/`git restore` of **your own** changes. Don't stack more fixes on a broken state.

## 6. WORKLOG.md (append-only activity log)

Log **everything**, including small actions: files read for decisions, commands run, experiments, failures, decisions, questions asked, and things you decided not to do.

- Append-only. Never edit or delete past entries; correct mistakes with a new entry.
- Newest entries at the bottom. Timestamp in UTC (`YYYY-MM-DD HH:MM`).
- Write an entry **before** starting a work unit (intent) and **after** it (result).
- Keep entries short and factual. Format:

```
### 2026-10-19 14:05 | feat/add-command | AI: <agent/model name>
Intent: implement `berth add` URL parsing
Actions: read src/source.rs; added parse_source(); ran cargo test
Result: 12 tests pass (output: `test result: ok. 12 passed`)
Commit: a1b2c3d
Noticed: asset scoring ignores .tar.zst (not fixed, out of scope)
Next: wire parse_source into CLI
```

- Humans may add entries too; mark them `Human:` instead of `AI:`.
- `WORKLOG.md` is the memory that survives context loss. Reread its tail at session start and after any long pause.

## 7. CHANGELOG.md (user-facing, curated)

- Follow Keep a Changelog: `Unreleased` section, categories `Added / Changed / Deprecated / Removed / Fixed / Security`.
- Add a line for every user-visible change in the same commit that makes it. Internal-only noise belongs in the worklog, not here.
- Don't create release sections or bump versions unless asked.

## 8. Safety rules

- **Needs explicit approval:** `rm -rf`, any recursive delete, `sudo`, `chmod -R`/`chown -R`, `curl | sh`, installing system packages, editing system/dotfiles outside the project, `dd`, disk/partition commands, database drops, `git push --force`.
- Stay inside the project directory. Never read or modify other projects, `~/.ssh`, browser profiles, password stores or key files.
- Never print, log or commit secrets. If you find one in the repo, flag it and don't copy it.
- Prefer dry-run modes (`--dry-run`, `-n`) first for anything that modifies state broadly.
- Use temp dirs and sandboxed `HOME`/XDG vars in tests. Tests must never touch the real home directory or real config.
- Network use: only HTTPS to official sources; don't pipe remote scripts to a shell; verify checksums when provided.

## 9. Code quality

- Match the existing style and structure. Read neighboring code first.
- Small functions, typed interfaces, clear names. Handle errors; no silent failure, no catch-all swallowing.
- Write or update tests with every behavior change. A bug fix needs a regression test.
- Run the project's formatter, linter, and tests after every change. If you can't find the commands, look in README/CI config; don't guess.
- Leave no debug prints, commented-out code, stray files, or temp artifacts. Clean up everything you create.
- Don't hide complexity: if a solution is hacky or partial, say so.

## 10. Context hygiene

- Maintain a short checklist for multi-step tasks and tick it as you go; record it in `WORKLOG.md`.
- When the session gets long or you feel unsure of earlier details, re-read `WORKLOG.md`, `git log` and the relevant files instead of relying on memory.
- Never "remember" file contents, command outputs, or decisions from earlier turns when you can cheaply re-check them.

## 11. Reporting format (end of every task)

```
Done:         what was completed (files, commit hashes)
Verified:     commands run + real results
Not verified: anything you couldn't test, and why
Assumptions:  anything unconfirmed
Noticed:      out-of-scope issues logged, not fixed
Next:         suggested next step (wait for approval)
```

Be concise. No filler, no hype, no claiming perfection. If something is wrong, say it plainly.

## 12. Human environment defaults

(Edit this section to taste.)

- OS: Fedora Linux (KDE Plasma), shell Bash. Linux-first; don't suggest Windows/macOS unless asked.
- Package order: DNF, Flatpak, AppImage, Nix, other repos last. No `sudo` actions without approval.
- Python: `uv`, `pyproject.toml` (hatchling), `src/` layout. Rust: stable via rustup, `cargo fmt`, `cargo clippy -- -D warnings`, `cargo nextest`.
- Config follows XDG: `~/.config/<app>/`, `~/.local/share/<app>/`, `~/.local/state/<app>/`, `~/.cache/<app>/`. **Never create dotfiles or junk in `$HOME`.**
- Dev uses AI heavily ("vibe coding"): explain non-obvious decisions briefly, don't over-explain basics.

## 13. Templates (create if missing)

`WORKLOG.md`: header explaining the append-only rule, then entries as in section 6.
`CHANGELOG.md`: Keep a Changelog header with an empty `Unreleased` section.
`.gitignore`: appropriate for the language, plus `.env`, `*.log`, build output, editor files.

---

# Project-specific rules: berth
berth is a Linux-only Rust CLI package manager for prebuilt binaries from GitHub/GitLab releases. **Read `SPEC.md` first. It is the source of truth.** If code and spec disagree, ask or update the spec explicitly; do not silently diverge.

## Workflow

- Work on **one phase at a time** from SPEC section 17. Do not start the next phase unprompted.
- Before coding a phase: restate the plan in a few bullets and list files you will touch.
- After each phase: all of these must pass before you report done:
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo nextest run` (or `cargo test` if nextest is unavailable)
- Make small, focused commits with clear messages (Conventional Commits).
- Never claim something works without running it. Show the commands you ran and their result.

## Architecture rules

- Cargo workspace: `crates/berth-core` (all logic, no printing) and `crates/berth` (clap parsing + terminal rendering only).
- Providers live behind the `Provider` trait. No provider-specific code outside its module.
- Pure logic (asset scoring, version compare, URL parsing) must be pure functions with no I/O, and unit tested.
- The install pipeline must be atomic (SPEC section 8). Never write directly into the final location; temp file, then rename on the same filesystem.
- All state changes go through the DB layer in transactions. Migrations are numbered and never edited after release.

## Code rules

- `#![forbid(unsafe_code)]`.
- No `unwrap()`, `expect()`, `panic!`, or indexing that can panic in non-test code. Use `thiserror` types in the core and `anyhow` context at the binary edge.
- No `println!` in `berth-core`. Return data; the CLI layer renders it (also needed for `--json`).
- Use `tracing` for logs; never log tokens or full auth headers.
- Prefer stable, well-maintained crates from SPEC section 14. Ask before adding any new dependency, and justify it in one line.
- Keep functions small and typed; avoid stringly-typed code (use enums/newtypes for provider, version, digest, etc.).

## Safety rules (non-negotiable)

- HTTPS only; reject HTTP and downgrade redirects.
- Verify checksums when available and hard-fail on mismatch; warn + record hash (TOFU) when unavailable.
- Never execute downloaded content. Never use `sudo`. Never write outside XDG dirs and the configured `bin_dir`.
- Refuse to overwrite files in `bin_dir` that berth does not manage unless `--force`.
- Default is **web mode (no API)**. API mode is opt-in per SPEC section 6.3.
- Scrapers must fail loudly on unexpected structure. Never guess assets.

## Testing rules

- Every bug fix gets a regression test.
- Scrapers are tested against saved fixtures in `tests/fixtures/` (do not hit the real network in normal tests).
- HTTP flows use `wiremock`. CLI behavior uses `assert_cmd` with a temp `HOME` and XDG dirs. Output snapshots use `insta`.
- Include failure-injection tests for the install pipeline (interrupt at each step; state must stay consistent).

## UX rules

- **Keep the CLI simple (SPEC section 4).** Do not add commands, flags, prompts or config keys that are not in the spec. Zero-config must work. Prefer smart defaults over options.

- Output style per SPEC section 13: step glyphs, spinners/bars via `indicatif`, plain output when not a TTY, `NO_COLOR` respected.
- Every user-facing error states what failed, why, and what to do next.
- Support `--json`, `--quiet`, `--verbose`, `--dry-run`, `--yes` consistently across commands.
- Use the exit codes defined in SPEC section 5.

## Things to ask the human about instead of guessing

- Changing a public command, flag, config key, DB schema, or exit code.
- Adding a dependency or raising the MSRV.
- Any scraper approach that depends on undocumented site behavior beyond what `docs/providers.md` records.
- Anything that would weaken a safety rule above.

## Environment

- Dev machine: Fedora Linux (KDE), Rust via rustup, shell Bash. Linux-only; do not add Windows/macOS code paths.
- Keep `docs/providers.md` and `CHANGELOG.md` updated as behavior changes.

- **Filesystem hygiene is a hard requirement (SPEC section 18):** only write to the XDG locations listed there, nothing in `$HOME` root, config at `~/.config/berth/berth.toml`, only delete files berth owns, clean up all temp files on every exit path, and ship the file-tree snapshot tests.
