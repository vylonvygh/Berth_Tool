# berth — Specification (v1)

> A fast, safe, user-level package manager for prebuilt binaries published on
> GitHub, GitLab and other git hosts. Linux only. Written in Rust.

```
berth add https://github.com/junegunn/fzf
berth update
```

## 1. Goals

- Replace the manual workflow: download release asset, extract, `chmod +x`, move to `~/.local/bin`, remember to check for updates.
- Feel like a real package manager: clear step-by-step output, safe, atomic, reversible.
- **No API by default.** Use web endpoints (redirects, Atom feeds, release pages). API access is an opt-in backend.
- Stable and boring: never leave a half-installed package, never corrupt state.

## 2. Non-goals (v1)

- Windows/macOS, system-wide installs, sudo/root.
- Building from source, dependency resolution between packages.
- Running install scripts or post-install hooks from upstream.
- Automatic/background updates (manual only in v1; systemd timer is a later feature).

## 3. Core concepts

- **Package**: a named binary tracked by berth (e.g. `fzf`), tied to a **source** (`github:junegunn/fzf`).
- **Source**: `provider:owner/repo` (GitLab: `gitlab:group/sub/project`). Provider is detected from the URL host.
- **Store**: real files live in `$XDG_DATA_HOME/berth/store/<name>/<version>/`.
- **Link**: `~/.local/bin/<name>` is a symlink into the store. Swapping the symlink is the atomic activate/rollback step.
- **State**: SQLite database in `$XDG_DATA_HOME/berth/berth.db`.
- **Config**: `$XDG_CONFIG_HOME/berth/berth.toml` (default `~/.config/berth/berth.toml`).
- **State/logs**: `$XDG_STATE_HOME/berth/` (default `~/.local/state/berth/`), for logs and the process lock.
- **Cache**: `$XDG_CACHE_HOME/berth/` (downloads, ETags, feed cache; always safe to delete).
- See section 18 for the filesystem hygiene rules.

## 4. Usability principles (simple by default)

1. **Five daily commands:** `add`, `remove`, `list`, `update`, `info`. Everything else is occasional or advanced.
2. **Zero config.** Works with no config file. No setup step, no `init`. First run just works.
3. **Paste anything.** `add` accepts whatever the user has in the clipboard (see 4.1); the user never has to learn berth's own syntax.
4. **Don't ask, just do** the obvious thing; ask only when genuinely ambiguous (e.g. two equally good assets), with a picker, never a wall of questions.
5. **Remember choices.** Asset pattern, executable path and prerelease choice are stored automatically, so `update` never asks again.
6. **Sensible short aliases:** `install`/`i` = `add`; `uninstall`/`rm` = `remove`; `ls` = `list`; `upgrade`/`up` = `update`.
7. **Helpful output:** each command ends with what happened and, if useful, one hint for the next step. Typos get "did you mean" suggestions. Errors say what to do next.
8. **Few flags.** Only five global flags are needed day to day: `-y/--yes`, `-n/--dry-run`, `-q/--quiet`, `-v/--verbose`, `--json`. Everything else is advanced and hidden from short `--help` (shown by `--help` long form).
9. **Safe by default, easy to undo.** `update` keeps the previous version, so `berth rollback <name>` is always one command.
10. **Running `berth` alone** prints a short status (packages installed, updates known from cache) and a 5-line cheat sheet.

### 4.1 Accepted `add` inputs

```
berth add https://github.com/junegunn/fzf            # repo URL
berth add github.com/junegunn/fzf                    # no scheme
berth add https://github.com/junegunn/fzf/releases/tag/v0.55.0   # release URL (pins that version)
berth add junegunn/fzf                               # shorthand, github by default
berth add gitlab:group/project                       # explicit provider
berth add https://gitlab.com/group/project
berth add junegunn/fzf BurntSushi/ripgrep sharkdp/fd # several at once
berth add junegunn/fzf@0.55.0                        # specific version
```

- Provider is detected from the host. Self-hosted hosts are registered in config (`[hosts."git.example.com"] type = "gitlab"`), or berth asks once and remembers.
- Already installed? Say so and suggest `berth update <name>`; don't error out.
- Rare overrides (advanced): `--name`, `--asset <regex>`, `--bin <path>`, `--pre`, `--pin`.

## 5. Commands

### Daily

| Command | What it does |
|---|---|
| `add <src>...` | Install one or more packages |
| `remove <name>...` | Remove packages completely (symlink, stored versions, DB rows) |
| `list` | Installed packages with version, source, and update status (from cache, no network) |
| `update [name...]` | Check for updates, show what's new, ask once "Upgrade N packages? [Y/n]", then upgrade. `--check` only checks (exit code `10` if updates exist). `-y` skips the question |
| `info <name>` | Source, installed and stored versions, hash, asset, install date |

### Occasional / advanced

| Command | What it does |
|---|---|
| `rollback <name> [version]` | Switch back to a previous stored version |
| `pin <name>` / `unpin <name>` | Hold a package at its current version (skipped by `update`) |
| `import` | Scan `~/.local/bin` for unmanaged executables and adopt them (asks for each source once) |
| `doctor` | Health check: broken symlinks, missing files, PATH, hash mismatches, scraper health. `--fix` repairs what berth provably owns. `--clean` removes orphans, stale cache and old versions (this replaces a separate `gc` command) |
| `self update` | Update berth itself |
| `self paths` | Print every directory and file berth uses, with disk usage |
| `self uninstall` | Remove all berth data, config, cache, state and managed symlinks (lists everything first, confirms; `--keep-binaries` converts links to real files first) |
| `self completions <shell>` | Generate bash/fish/zsh completions |
| `config` | Show config path and values (`config get/set <key>`) |

Cleanup of old versions and temp files also happens **automatically** after `add`, `update` and `remove`; `doctor --clean` exists for manual or recovery use.

Global flags: `-y`, `-n/--dry-run`, `-q`, `-v`, `--json`. Advanced/hidden: `--api`, `--no-color`.

Exit codes: `0` ok, `1` generic error, `2` usage error, `3` network error, `4` verification failure, `5` nothing to do / no match, `10` updates available (`update --check`, for scripting).

## 6. Provider layer

```rust
trait Provider {
    async fn latest(&self, repo: &RepoRef, opts: &ResolveOpts) -> Result<Release>;
    async fn release(&self, repo: &RepoRef, tag: &str) -> Result<Release>;
    async fn list_releases(&self, repo: &RepoRef, limit: usize) -> Result<Vec<ReleaseSummary>>;
}
struct Release { tag: String, version: Version, published: Option<DateTime>, prerelease: bool, assets: Vec<Asset> }
struct Asset { name: String, url: Url, size: Option<u64>, digest: Option<Digest> }
```

Backends: `github_web`, `github_api`, `gitlab_web`, `gitlab_api`, `forgejo_api`. Each lives in its own module behind the trait.

### 6.1 GitHub web mode (default)

- Latest tag: request `/<o>/<r>/releases/latest`, do not follow the redirect, read the `Location` header to extract the tag.
- Version history / prerelease detection: `/<o>/<r>/releases.atom`.
- Asset list: fetch `/<o>/<r>/releases/expanded_assets/<tag>` and parse HTML (use `scraper` crate). Extract asset name, URL, size, and digest if present.
- Download URL: `/<o>/<r>/releases/download/<tag>/<asset>`.
- Conditional requests with ETag / If-None-Match for atom feeds.

### 6.2 GitLab

- Prefer the GitLab public API for GitLab hosts if web scraping proves unreliable; the unauthenticated read limits are generous. Decide in Phase 0 research (see section 14) and document the result in `docs/providers.md`.
- Latest: `/-/releases/permalink/latest` redirect where available; else API.

### 6.3 API mode (opt-in)

- Enabled per host in config or with `--api`.
- Reads token from `BERTH_GITHUB_TOKEN` / `GITHUB_TOKEN` / `BERTH_GITLAB_TOKEN`, or a `0600` token file in the config dir. Never log tokens.
- Provides digests, stable JSON, and is the automatic suggestion when a web scraper fails.

### 6.4 Scraper safety rules

- Every scraper is isolated in its own module with HTML/XML fixtures in `tests/fixtures/`.
- If parsing yields zero assets or an unexpected structure, fail with: `scraper failed for <host>; try --api` — never guess.
- Weekly CI canary job hits a few real public repos and fails loudly if parsing breaks.
- Set a descriptive `User-Agent: berth/<version>`. Respect `Retry-After`. Cap concurrency (default 6).

## 7. Asset selection

Deterministic scoring function (`select_asset(assets, platform) -> Ranked<Asset>`), pure and heavily unit-tested.

1. Hard exclude: checksums/signatures (`.sha256`, `.sha512`, `.sig`, `.asc`, `.pem`, `.sbom`, `.intoto.jsonl`), packages for other managers (`.deb`, `.rpm`, `.apk`, `.msi`, `.exe`, `.dmg`, `.pkg`, `.AppImage` unless allowed), source archives, other OS (`darwin`, `macos`, `windows`, `win32`, `freebsd`...), other arch.
2. Score: matches OS `linux` (+), matches arch aliases (`x86_64`, `amd64`, `x64`, `aarch64`, `arm64`) (+), `musl`/`static` (+ small), `gnu` (neutral), archive formats `tar.gz`/`tar.xz`/`tar.zst`/`zip`/raw (all accepted), name contains package name (+).
3. If top score ties or is below a threshold: interactive picker (TTY) or error with candidates (non-TTY / `--yes` picks top only if unambiguous margin).
4. A user override `--asset <regex>` is stored in the DB and reused on updates.
5. Executable discovery inside archives: prefer file named like the package, executable bit set, in root or `bin/`; else prompt.

## 8. Install pipeline (atomic)

1. Parse input, detect provider, acquire process lock.
2. Resolve release (tag, assets). Skip prereleases unless `--pre`.
3. Select asset (section 7).
4. Download to `cache/downloads/<random>.part`, with progress and size check; resume is optional later.
5. Verify (section 9).
6. Extract into a temp dir inside the store filesystem; locate executable; `chmod 0755`.
7. Move to `store/<name>/<version>/` (rename, same filesystem).
8. Create new symlink as `<name>.tmp` in the bin dir, then `rename` over `<name>` (atomic replace). Refuse to overwrite a non-berth file at that path unless `--force`.
9. Write DB record in a single transaction; on any earlier failure nothing is activated and temp files are removed.
10. Warn if `bin_dir` is not on `PATH`.

## 9. Verification policy

- If a checksums file or asset digest is available: **verify sha256 (or stated algorithm); hard-fail on mismatch.**
- If none is available: **warn**, then record the SHA-256 of the downloaded asset (TOFU). On later updates of the same package, compare nothing across versions, but `doctor` re-verifies the stored binary against its recorded hash.
- HTTPS only; reject plain HTTP and downgrading redirects.
- Never execute downloaded content during install.
- Show source, version, asset name, and hash before activating (always; `--yes` skips only the confirmation prompt).

## 10. Updates

- `update` (no args) and `update --check`: for each non-pinned package fetch latest (parallel, bounded), compare, print a table (name, installed, latest, what changed), exit code `10` if any are outdated and `--check` was used.
- Version compare: lenient semver (strip leading `v`, tolerate 2-part versions); fallback to published date if tags are not semver.
- Prereleases ignored by default; per-package opt-in `berth pin`-like flag `--pre`.
- `update` then asks one confirmation (`-y` skips) and runs the install pipeline for each outdated package; failures don't abort the rest; summary at end with per-package result.
- Keep previous version(s) per `keep_versions` (default 2) for rollback.
- Manual only in v1.

## 11. Data model (SQLite, `rusqlite` bundled)

```
meta(key, value)                      -- schema_version
packages(id, name UNIQUE, provider, host, repo_path, asset_regex, bin_path,
         pinned, allow_prerelease, active_version, installed_at, updated_at)
versions(id, package_id, version, tag, asset_name, sha256, size,
         verified, source_url, installed_at)
http_cache(url PRIMARY KEY, etag, last_modified, body_hash, fetched_at)
```

- Schema migrations are numbered and applied on startup inside a transaction; refuse to run on a newer unknown schema.
- WAL mode; single-writer guaranteed by the process lock (`fd-lock` on `$XDG_RUNTIME_DIR/berth.lock` or data dir fallback).

## 12. Config (`berth.toml`)

```toml
bin_dir = "~/.local/bin"
keep_versions = 2
concurrency = 6
color = "auto"          # auto | always | never

[defaults]
prerelease = false
mode = "web"            # web | api

[hosts."github.com"]
mode = "web"

[hosts."gitlab.com"]
mode = "web"
```

Precedence: CLI flags > env vars > config file > defaults. No `.env` in production.

## 13. Terminal UX

- Use `indicatif` (spinners, bars) + `console`/`owo-colors`. Steps use a consistent glyph set: `✔` done, `⠙` running, `!` warning, `✘` error. ASCII fallback if the locale is not UTF-8.
- Non-TTY: plain line output, no ANSI; `NO_COLOR` respected; `--json` emits one JSON document (or NDJSON for streams) with no decoration.
- Example:

```
$ berth add junegunn/fzf
 ✔ Resolved    github:junegunn/fzf → v0.55.0
 ✔ Selected    fzf-0.55.0-linux_amd64.tar.gz (3.4 MiB)
 ⠙ Downloading [████████░░] 2.1/3.4 MiB
 ✔ Verified    sha256 ok
 ✔ Installed   fzf 0.55.0 → ~/.local/bin/fzf
```

- Bare `berth`:

```
$ berth
berth 0.1.0 · 14 packages · 3 updates available (checked 2h ago)

  add <repo-url>     install a tool       update   upgrade everything
  list               see what you have    remove   uninstall a tool
  info <name>        details              doctor   check health
```

- Errors: what failed, why, and the next step; no raw backtraces unless `-v`.

## 14. Tech stack

| Area | Crate |
|---|---|
| CLI | `clap` (derive), `clap_complete`, `clap_mangen` |
| Async/HTTP | `tokio`, `reqwest` (rustls, no OpenSSL, gzip, redirects off for latest-tag probing) |
| HTML/XML | `scraper`, `quick-xml` (or `feed-rs`) |
| Serialization | `serde`, `serde_json`, `toml` |
| DB | `rusqlite` (bundled) |
| Versions | `semver`, `regex` |
| Hashing | `sha2`, `hex` |
| Archives | `tar`, `flate2`, `liblzma` (or `xz2`), `zstd`, `zip` |
| UX | `indicatif`, `console`, `inquire`, `comfy-table` |
| Paths | `directories` |
| Errors/logging | `thiserror` (lib), `anyhow` (bin), `tracing`, `tracing-subscriber` |
| Locking/FS | `fd-lock`, `tempfile` |

Edition 2021+, MSRV pinned in `Cargo.toml`. Workspace: `crates/berth-core` (library, all logic) and `crates/berth` (thin CLI).

## 15. Quality bar

- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` (pedantic where sensible), `cargo deny` / `cargo audit` in CI.
- No `unwrap`/`expect` in non-test code; `#![forbid(unsafe_code)]`.
- Tests: unit (asset scoring, version compare, URL parsing), fixture tests for scrapers, `wiremock` for HTTP flows, `assert_cmd` + `tempfile` for end-to-end CLI tests in a sandboxed `HOME`/XDG, `insta` snapshots for output. Run with `cargo nextest`.
- Property tests (`proptest`) for URL parsing and version comparison.
- Crash safety tests: kill mid-download, mid-extract, mid-link; state must stay consistent.

## 16. Release & distribution

- GitHub Actions: build static `x86_64-unknown-linux-musl` and `aarch64-unknown-linux-musl`, publish with checksums (consider `cargo-dist`).
- Man page and shell completions generated at build time and shipped in the archive.
- `berth self-update` uses berth's own provider logic.
- Later: Fedora COPR package, publish to crates.io.
- Repo hygiene: LICENSE (MIT or Apache-2.0), README, CHANGELOG (Keep a Changelog), SECURITY.md, CONTRIBUTING.md, semver releases.

## 17. Build phases (each ends green: fmt, clippy, tests)

0. **Research**: `curl` the real GitHub/GitLab endpoints from section 6, save fixtures, write `docs/providers.md` with findings and the final web-vs-API decision per provider.
1. Workspace skeleton, config loading, XDG paths, logging, DB + migrations, process lock.
2. URL/source parsing, `Provider` trait, GitHub web backend with fixtures.
3. Asset scoring + executable discovery (largest test suite).
4. Download, verify, extract, atomic install, symlinks; `add`, `remove`, `list`.
5. `update` (with `--check`), ETag cache, `pin`/`unpin`, bare `berth` status screen.
6. `rollback`, `info`, `import`, `doctor` (with `--fix`/`--clean`), `self paths`, `self uninstall`.
7. GitLab backend (+ Forgejo), API mode, host config.
8. UX polish, `--json`, completions, man page, CI, release pipeline, `self-update`.

## 18. Filesystem hygiene (no garbage, clean home)

**Where berth may write (and nowhere else):**

| Purpose | Path (XDG, honoring `XDG_*` env vars) |
|---|---|
| Config | `~/.config/berth/berth.toml` |
| Data (store, DB) | `~/.local/share/berth/` |
| State (logs, lock) | `~/.local/state/berth/` |
| Cache (downloads, feeds) | `~/.cache/berth/` |
| Installed links | `bin_dir` (default `~/.local/bin`), symlinks only |

- **Nothing in `$HOME` root.** No `~/.berth`, no stray files, no dotfiles. Resolve all paths via the `directories` crate; fall back to the XDG defaults above.
- **Ownership tracking:** berth only deletes what it created. A file is deleted only if it is inside berth's own directories, or it is a symlink in `bin_dir` recorded in the DB whose target is inside the store. Never delete user files or foreign binaries.
- **Temp files:** all in-progress work lives in `~/.cache/berth/tmp/` (same filesystem as the target for atomic rename). Use RAII guards (`tempfile`) so they are removed on success, error, and panic. On startup, sweep stale `*.part` and temp dirs older than 24h left by a hard kill (SIGKILL, power loss).
- **Cache policy:** downloads are deleted right after a successful install (keep only with `cache.keep_downloads = true`). Feed/ETag cache has a TTL and size cap; `doctor --clean` and automatic cleanup prune it.
- **`remove <name>`:** deletes the symlink, all stored versions, and DB rows, then removes now-empty parent directories (only inside berth dirs). `--keep-versions` is the only exception.
- **Version pruning:** keep `keep_versions` per package (default 2); older versions are deleted automatically after a successful update.
- **`doctor --clean`** (and automatic cleanup after add/update/remove): removes orphaned store directories not in the DB, dangling berth-owned symlinks, stale temp/cache files, and reports bytes freed. Supports `--dry-run`.
- **`doctor`:** reports (does not silently fix) mismatches: symlink pointing elsewhere, missing store files, DB rows without files, files without DB rows. `doctor --fix` repairs only what it can prove berth owns.
- **`self uninstall`:** lists everything it will delete (config, data, state, cache, managed symlinks), asks for confirmation (`--yes` to skip), then removes it all, leaving zero berth files behind. Installed binaries are removed too unless `--keep-binaries` (which first replaces symlinks with real copies in `bin_dir`).
- **`self paths`:** prints every location above plus disk usage, so the user can always see exactly what berth owns.
- **Config is never auto-created with junk:** `berth.toml` is only written by `berth config init` or when the user changes a setting; defaults work with no file present.
- **Logs:** written to `~/.local/state/berth/berth.log`, rotated (size-capped, e.g. 1 MiB x 3). Nothing is logged to stdout unless `-v`.

**Tests (required):**
- After every command in the e2e suite, snapshot the full file tree of a sandboxed `HOME`; assert only the five locations above exist and no `.part`/temp files remain.
- After `remove` of the last package and after `self uninstall`, assert the sandbox `HOME` tree is identical to its pre-install state.
- Crash tests: kill mid-download/extract/link, run any command, assert stale files are swept and state is consistent.
