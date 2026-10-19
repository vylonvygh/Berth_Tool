# Provider research (Phase 0)

Findings from probing the real endpoints described in `SPEC.md` section 6.
This document is the source of truth for how each provider backend talks to
its host. Raw responses are saved under `tests/fixtures/`.

## Method

- Date: 2026-10-19 (UTC).
- Tool: `curl 8.x` over HTTPS only, `User-Agent: berth/0.1.0`, **unauthenticated**.
- Redirect probes were run **without** `-L` so the `Location` header is visible.
- Every claim below is marked `VERIFIED` (with the evidence) or `ASSUMPTION` / `UNKNOWN`.
- Reference repos: GitHub `junegunn/fzf`, `neovim/neovim`, `torvalds/linux`;
  GitLab `gitlab-org/cli`.
- The API host is in the response headers (`server`, `x-github-*`, `ratelimit-*`);
  a regional edge memory (`centralindia`) and `server: fasthttp` were observed on
  api.github.com. Treated as normal GitHub infrastructure, not a proxy we control.

## GitHub

### Web mode (default)

| Step | Request | Observed | Evidence |
|---|---|---|---|
| Latest tag | `GET https://github.com/{o}/{r}/releases/latest` | `302`, `Location: .../releases/tag/{tag}` | `github/fzf-releases-latest.headers` → `.../tag/v0.74.5` |
| No releases | same, repo with tags but no releases | `302`, `Location: .../releases` (**no** `/tag/`) | `github/linux-releases-latest.headers` (torvalds/linux) |
| Version history | `GET /{o}/{r}/releases.atom` | `200 application/atom+xml`, weak `ETag` | `github/fzf-releases.atom`, `.headers` |
| Conditional | `GET` atom with `If-None-Match: <etag>` | `304` | observed live; `github/fzf-atom-304.headers` |
| Assets | `GET /{o}/{r}/releases/expanded_assets/{tag}` | `200 text/html` | `github/fzf-expanded_assets.html` |
| Download | `GET /{o}/{r}/releases/download/{tag}/{asset}` | `302` → `https://release-assets.githubusercontent.com/...` | `github/fzf-download-redirect.headers` |

VERIFIED details:

- **Latest tag** comes from the `Location` header of `releases/latest`. Do not follow
  the redirect; parse the last path segment after `/tag/`.
- **"Repo with no releases" is not a 404.** GitHub still returns `302` but points at
  `/releases` (no `/tag/<tag>`). Detect "no releases" by the absence of `/tag/` in
  the `Location` path. VERIFIED with `torvalds/linux`.
- **Atom feed**: entries carry `id`, `updated`, `<link rel="alternate" href=".../releases/tag/{tag}">`,
  `<title>` (the tag with the leading `v` stripped in GitHub's rendering, e.g. `0.74.5`), and
  `<content type="html">` release notes. VERIFIED.
- **Atom includes prereleases.** `neovim/neovim` atom's newest entry is
  "Nvim development (prerelease) build" while `releases/latest` resolves to the stable
  `v0.12.6`. VERIFIED (`github/neovim-releases.atom`, `github/neovim-releases-latest.headers`).
- **Atom has no structured prerelease flag.** No field on the entry distinguishes a
  prerelease; only free-text titles (repo-specific) hint at it. VERIFIED (inspected `fzf` and
  `neovim` feeds). See "Open questions" #1.
- **`expanded_assets/{tag}`** returns a small HTML fragment (no `<html>`/`<body>` wrapper).
  Per asset it contains:
  - name: `<span class="text-bold">{asset}</span>`
  - href: a **relative** path `/owner/repo/releases/download/{tag}/{asset}` (resolve against `https://github.com`)
  - digest: free text `sha256:<64 hex>` (present for fzf assets)
  - size: human string such as `2.18 MB` (e.g. `2.25 MB`) — needs parsing to bytes.
  VERIFIED (`github/fzf-expanded_assets.html`, 52 `sha256:` occurrences).
- **Download URL** redirects to `release-assets.githubusercontent.com` — **HTTPS → HTTPS**,
  no downgrade. VERIFIED.
- **Prerelease badge** on the release page `/{o}/{r}/releases/tag/{tag}`:
  the word `Pre-release` appears for a prerelease; `Latest` appears for the latest stable.
  VERIFIED (`github/neovim-nightly-tag-page.html` → 2× `Pre-release`;
  `github/neovim-v0126-tag-page.html` → 2× `Latest`). Costs one HTML fetch per tag.
- **Rate limits:** github.com web pages carried **no** `x-ratelimit-*` headers in these
  responses. VERIFIED.

### API mode (opt-in, section 6.3)

- `GET https://api.github.com/repos/{o}/{r}/releases/latest` → `200` JSON.
- Each asset carries `digest` = `sha256:<64 hex>`. VERIFIED (`github/api-latest-fzf.json`;
  first digest `sha256:3116a74c...` matches the android_arm64 asset in `expanded_assets`).
- Unauthenticated rate limit: `x-ratelimit-limit: 60` per hour. VERIFIED
  (`github/api-rate-limit.headers`, `github/api-latest-fzf.headers`).

## GitLab

| Step | Request | Observed | Evidence |
|---|---|---|---|
| Latest (web) | `GET https://gitlab.com/{group}/{project}/-/releases/permalink/latest` | `302`, `Location: .../-/releases/{tag}` | `gitlab/cli-permalink-latest.headers` → `v1.123.0` |
| Atom (web) | `GET /-/releases.atom` | `200`, 162 entries, titles = tags, **no asset data** | observed live (fixture pruned, see WORKLOG 08:2x) |
| Release page (web) | `GET /-/releases/{tag}` | `200` HTML, **assets are client-side only** | `gitlab/cli-release-v1.123.0.html` (0 asset names / download links / checksums) |
| Latest (API) | `GET /api/v4/projects/{urlencoded}/releases/permalink/latest` | `302` → **relative** `/api/v4/projects/{id}/releases/{tag}` | `gitlab/api-permalink-latest.headers`, `...-followed.json` |
| Releases (API) | `GET /api/v4/projects/{urlencoded}/releases?per_page=N` | `200` JSON | `gitlab/api-releases.json` |
| Download | `GET` asset `url` (packages/generic) | `200 application/gzip`, `content-length`, `ETag`, `content-disposition` | observed live |

VERIFIED details:

- **Web permalink redirect works** and yields the latest tag in the `Location` path.
  Multi-segment groups (`group/sub/project`) must be handled by the URL parser.
- **GitLab web release pages cannot be scraped for assets.** The HTML is a shell; asset
  names, download links and checksums are not present server-side. VERIFIED (0 matches for
  asset names / `downloads` hrefs / `sha256` in the fixture). This is decisive for the
  web-vs-API decision.
- **GitLab API permalink `Location` is relative** (`/api/v4/...`), unlike GitHub. The client
  must resolve it against the host, or call the canonical project-ID URL.
- **API release JSON** includes `tag_name`, `released_at`, `upcoming_release` (GitLab's
  prerelease marker), `assets.links[]` with `{id, name, url, direct_asset_url, link_type}`,
  and `assets.sources[]` (auto zip/tar.gz archives). `direct_asset_url` uses the human
  pattern `/-/releases/{tag}/downloads/{name}`. VERIFIED.
- **No digest field.** `assets.links[]` and `assets.sources[]` carry **no checksum**
  (0 `"digest"` keys in `api-releases.json`). So GitLab assets are TOFU unless the release
  itself ships a checksums file as an asset. VERIFIED.
- **Unauthenticated API rate limit:** `ratelimit-limit: 500` (`throttle_unauthenticated_api`),
  with `ratelimit-remaining` / `ratelimit-reset`. VERIFIED (`gitlab/api-releases.headers`).
  Generous enough for normal use.
- **Downloads do not redirect** for the `packages/generic` URL: `200` with
  `application/gzip`, `content-length: 18445062`, an `ETag`, and a
  `content-disposition: attachment` filename. VERIFIED (HEAD request).

## Decisions

1. **GitHub → web mode by default**, exactly as section 6.1:
   `releases/latest` (redirect, no follow) for the latest tag, `releases.atom`
   (+ `If-None-Match`/ETag) for history, `expanded_assets/{tag}` (HTML scrape) for assets
   with sha256 + size, and `releases/download/...` (redirect) for the file. API mode is
   opt-in for digests and reliability.
2. **GitLab → use the public GitLab REST API, unauthenticated, as the *default*.**
   Rationale: the web release page does not expose assets at all, so HTML scraping is
   impossible; the unauthenticated API is structured, gives the prerelease flag, and has a
   500/window limit. This does not need a token and is therefore still "no setup".
   **This needs a SPEC clarification** (see Open questions #2): SPEC §6 frames "web mode" as
   "no API". For GitLab, the practical reading is "the public API without a token". Asset
   digests are absent, so the section 9 TOFU policy applies.
3. **Forgejo:** not probed (section 17 places it in Phase 7; section 6 has no Forgejo
   subsection). UNKNOWN.

## Fixtures

`tests/fixtures/github/`: fzf `releases/latest` + atom + atom ETag headers + 304 response,
fzf `expanded_assets` HTML + headers, fzf download-redirect headers, fzf `releases.atom`,
linux (no-releases) `releases/latest` headers, neovim atom + `releases/latest` headers,
neovim prerelease/stable tag pages, GitHub API `releases/latest` JSON + headers,
`rate_limit` JSON + headers.

`tests/fixtures/gitlab/`: permalink-latest headers, API permalink headers + followed JSON,
API releases list JSON + headers, client-side release page HTML.

Two oversized probes (a 1.8 MB GitLab atom and a 544 KB GitHub releases-list page) were
deleted after analysis to keep the repo lean; both findings are recorded above.

## Open questions (need a human decision)

1. **GitHub prerelease detection.** SPEC §6.1 says use the atom feed, but the atom feed has no
   structured prerelease flag. Proposed: fetch the tag page and look for the `Pre-release`
   badge only when needed (e.g. the atom's newest tag differs from `releases/latest`), or
   always fetch the one release page. Confirm the approach.
2. **GitLab "web mode" wording.** Confirm that for GitLab, "web mode" may mean "public
   unauthenticated REST API (no token)" — otherwise GitLab cannot list assets at all.
3. **GitLab digests are unavailable** → TOFU (record sha256 of the download) unless the
   release ships a checksums asset. Confirm this is acceptable.
4. **XML parser choice** for the atom feeds (`feed-rs` vs `quick-xml`): SPEC §14 allows
   either. Decide in Phase 2.
5. **Forgejo** endpoints are undocumented in SPEC §6; to be researched in Phase 7.
