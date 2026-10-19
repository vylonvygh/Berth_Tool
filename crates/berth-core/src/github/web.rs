use crate::error::{Error, Result};
use crate::github::types::AssetRow;
use crate::provider::{Provider, ResolveOpts};
use crate::types::{Asset, Digest, Release, ReleaseSummary, RepoRef, Version};

pub struct GitHubWeb {
    client: reqwest::Client,
    user_agent: String,
    web_base: String,
}

pub fn is_loopback(url: &url::Url) -> bool {
    matches!(
        url.host_str(),
        Some("localhost") | Some("127.0.0.1") | Some("[::1]") | Some("::1")
    )
}

fn secure_url(url: &url::Url) -> bool {
    url.scheme() == "https" || (url.scheme() == "http" && is_loopback(url))
}

impl GitHubWeb {
    pub fn new(user_agent: impl Into<String>) -> Result<Self> {
        Self::with_base(user_agent, "https://github.com")
    }

    pub fn with_base(user_agent: impl Into<String>, web_base: impl Into<String>) -> Result<Self> {
        let base = url::Url::parse(&web_base.into()).map_err(|e| Error::Parse {
            what: "web base url".to_string(),
            message: e.to_string(),
        })?;
        if !secure_url(&base) {
            return Err(Error::Provider(format!(
                "refusing non-https web base {base}"
            )));
        }
        let client = reqwest::Client::builder()
            .use_rustls_tls()
            .redirect(reqwest::redirect::Policy::custom(|attempt| {
                if !secure_url(attempt.url()) {
                    attempt.error("refusing insecure redirect")
                } else if attempt.previous().len() > 10 {
                    attempt.error("too many redirects")
                } else {
                    attempt.follow()
                }
            }))
            .build()
            .map_err(|source| Error::Http {
                url: String::new(),
                source,
            })?;
        Ok(Self {
            client,
            user_agent: user_agent.into(),
            web_base: base.to_string().trim_end_matches('/').to_string(),
        })
    }

    fn latest_url(&self, repo: &RepoRef) -> String {
        format!(
            "{}/{}/{}/releases/latest",
            self.web_base, repo.owner, repo.repo
        )
    }

    fn atom_url(&self, repo: &RepoRef) -> String {
        format!(
            "{}/{}/{}/releases.atom",
            self.web_base, repo.owner, repo.repo
        )
    }

    fn expanded_assets_url(&self, repo: &RepoRef, tag: &str) -> String {
        format!(
            "{}/{}/{}/releases/expanded_assets/{}",
            self.web_base, repo.owner, repo.repo, tag
        )
    }

    async fn get_text(&self, url: &str) -> Result<String> {
        let resp = self
            .client
            .get(url)
            .header(reqwest::header::USER_AGENT, &self.user_agent)
            .send()
            .await
            .map_err(|source| Error::Http {
                url: url.to_string(),
                source,
            })?;
        let resp = resp.error_for_status().map_err(|source| Error::Http {
            url: url.to_string(),
            source,
        })?;
        resp.text().await.map_err(|source| Error::Http {
            url: url.to_string(),
            source,
        })
    }
}

fn selector(s: &str) -> Result<scraper::Selector> {
    scraper::Selector::parse(s).map_err(|e| Error::Parse {
        what: format!("css selector {s:?}"),
        message: e.to_string(),
    })
}

fn human_size_to_bytes(text: &str) -> Option<u64> {
    let re = regex::Regex::new(r"^(\d+(?:\.\d+)?)\s*(B|KB|MB|GB|TB)$").ok()?;
    let caps = re.captures(text.trim())?;
    let num: f64 = caps[1].parse().ok()?;
    let mul = match &caps[2] {
        "B" => 1.0,
        "KB" => 1024.0,
        "MB" => 1024.0 * 1024.0,
        "GB" => 1024.0 * 1024.0 * 1024.0,
        "TB" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => return None,
    };
    Some((num * mul) as u64)
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = &s[i + 1..i + 3];
            if let Ok(b) = u8::from_str_radix(hex, 16) {
                out.push(b);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

pub fn version_from_tag(tag: &str) -> Version {
    let stripped = tag
        .strip_prefix('v')
        .or(tag.strip_prefix('V'))
        .unwrap_or(tag);
    match semver::Version::parse(stripped) {
        Ok(v) => Version::Semver(v),
        Err(_) => Version::Tag(tag.to_string()),
    }
}

pub fn header_value(raw_headers: &str, name: &str) -> Option<String> {
    let name = name.to_ascii_lowercase();
    for line in raw_headers.lines() {
        if let Some((key, value)) = line.split_once(':') {
            if key.trim().eq_ignore_ascii_case(&name) {
                return Some(value.trim().to_string());
            }
        }
    }
    None
}

pub fn latest_tag_from_location(location: &str, repo: &RepoRef) -> Result<String> {
    let url = url::Url::parse(location).map_err(|e| Error::Parse {
        what: format!("redirect location {location:?}"),
        message: e.to_string(),
    })?;
    if !secure_url(&url) {
        return Err(Error::Provider(format!(
            "insecure redirect location {location}"
        )));
    }
    let prefix = format!("/{}/{}/releases/", repo.owner, repo.repo);
    let plain = format!("/{}/{}/releases", repo.owner, repo.repo);
    let path = url.path();
    if path == plain {
        return Err(Error::Provider(format!(
            "no releases published for {}/{}",
            repo.owner, repo.repo
        )));
    }
    let rest = path.strip_prefix(&prefix).ok_or_else(|| {
        Error::Provider(format!(
            "unexpected redirect location {location} (expected {prefix}tag/...)"
        ))
    })?;
    if rest == "tag" || !rest.starts_with("tag/") {
        return Err(Error::Provider(format!(
            "no releases published for {}/{}",
            repo.owner, repo.repo
        )));
    }
    Ok(percent_decode(&rest["tag/".len()..]))
}

pub fn tag_from_link(link: &str) -> Option<String> {
    let marker = "/releases/tag/";
    let idx = link.find(marker)?;
    Some(percent_decode(&link[idx + marker.len()..]))
}

impl GitHubWeb {
    pub fn parse_expanded_assets(html: &str) -> Result<Vec<AssetRow>> {
        let doc = scraper::Html::parse_document(html);
        let anchor_sel = selector("a[href*='/releases/download/']")?;
        let name_sel = selector("span.text-bold")?;
        let span_sel = selector("span")?;
        let clip_sel = selector("clipboard-copy[aria-label^='Copy to clipboard digest for']")?;
        let time_sel = selector("relative-time[datetime]")?;

        let mut names: Vec<String> = Vec::new();
        let mut hrefs: Vec<String> = Vec::new();
        for a in doc.select(&anchor_sel) {
            if let Some(href) = a.value().attr("href") {
                let name = a
                    .select(&name_sel)
                    .next()
                    .map(|el| el.text().collect::<Vec<_>>().join(""))
                    .unwrap_or_default();
                names.push(name);
                hrefs.push(href.to_string());
            }
        }
        if names.is_empty() {
            return Err(Error::Parse {
                what: "expanded_assets page".to_string(),
                message: "no release download links found".to_string(),
            });
        }

        let mut sizes: Vec<Option<u64>> = Vec::new();
        let mut published: Vec<Option<String>> = Vec::new();
        for el in doc.select(&span_sel) {
            let text = el.text().collect::<Vec<_>>().join("");
            if let Some(bytes) = human_size_to_bytes(&text) {
                sizes.push(Some(bytes));
            }
        }
        for el in doc.select(&time_sel) {
            if let Some(dt) = el.value().attr("datetime") {
                published.push(Some(dt.to_string()));
            }
        }

        let mut digests: Vec<Option<String>> = vec![None; names.len()];
        for el in doc.select(&clip_sel) {
            let label = el.value().attr("aria-label").unwrap_or_default();
            let value = el.value().attr("value").unwrap_or_default();
            let name = label
                .strip_prefix("Copy to clipboard digest for ")
                .unwrap_or_default();
            if let Some(pos) = names.iter().position(|n| n == name) {
                digests[pos] = Some(value.strip_prefix("sha256:").unwrap_or(value).to_string());
            }
        }

        let mut rows = Vec::with_capacity(names.len());
        for i in 0..names.len() {
            rows.push(AssetRow {
                name: names[i].clone(),
                url_relative: hrefs[i].clone(),
                size: sizes.get(i).copied().flatten(),
                sha256: digests[i].clone(),
                published: published.get(i).cloned().flatten(),
            });
        }
        Ok(rows)
    }

    pub fn parse_atom(xml: &str) -> Result<Vec<AtomEntry>> {
        let mut entries = Vec::new();
        let mut reader = quick_xml::Reader::from_str(xml);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();
        let mut in_entry = false;
        let mut current_tag: Option<String> = None;
        let mut id = String::new();
        let mut title = String::new();
        let mut link: Option<String> = None;
        let mut updated: Option<String> = None;
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(quick_xml::events::Event::Start(e)) => {
                    let name = e.name();
                    if name.as_ref() == b"entry" {
                        in_entry = true;
                        id.clear();
                        title.clear();
                        link = None;
                        updated = None;
                        current_tag = None;
                    } else if in_entry {
                        current_tag = Some(String::from_utf8_lossy(name.as_ref()).into_owned());
                    }
                }
                Ok(quick_xml::events::Event::Empty(e)) => {
                    let name = e.name();
                    if in_entry && name.as_ref() == b"link" {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"href" {
                                link = Some(String::from_utf8_lossy(&attr.value).into_owned());
                            }
                        }
                    }
                }
                Ok(quick_xml::events::Event::End(e)) => {
                    let name = e.name();
                    if name.as_ref() == b"entry" {
                        in_entry = false;
                        entries.push(AtomEntry {
                            id: id.clone(),
                            title: title.clone(),
                            link: link.clone(),
                            updated: updated.clone(),
                        });
                    } else if in_entry {
                        current_tag = None;
                    }
                }
                Ok(quick_xml::events::Event::Text(e)) => {
                    if in_entry {
                        if let Ok(txt) = e.unescape() {
                            let t = txt.to_string();
                            match current_tag.as_deref() {
                                Some("id") => id = t,
                                Some("title") => title = t,
                                Some("updated") => updated = Some(t),
                                _ => {}
                            }
                        }
                    }
                }
                Ok(quick_xml::events::Event::Eof) => break,
                Err(e) => {
                    return Err(Error::Parse {
                        what: "atom feed".to_string(),
                        message: e.to_string(),
                    })
                }
                _ => {}
            }
            buf.clear();
        }
        if in_entry {
            return Err(Error::Parse {
                what: "atom feed".to_string(),
                message: "unclosed <entry> element".to_string(),
            });
        }
        Ok(entries)
    }
}

#[derive(Debug, Clone)]
pub struct AtomEntry {
    pub id: String,
    pub title: String,
    pub link: Option<String>,
    pub updated: Option<String>,
}

impl AtomEntry {
    pub fn tag(&self) -> Option<String> {
        self.link.as_deref().and_then(tag_from_link)
    }
}

pub fn rows_to_assets(rows: &[AssetRow], host: &str) -> Result<Vec<Asset>> {
    rows.iter()
        .map(|row| {
            let url = if row.url_relative.starts_with("https://") {
                url::Url::parse(&row.url_relative)
            } else if let Some(rest) = row.url_relative.strip_prefix('/') {
                url::Url::parse(&format!("https://{host}/{rest}"))
            } else {
                url::Url::parse(&format!("https://{host}/{}", row.url_relative))
            }
            .map_err(|e| Error::Parse {
                what: format!("asset url {:?}", row.url_relative),
                message: e.to_string(),
            })?;
            Ok(Asset {
                name: row.name.clone(),
                url,
                size: row.size,
                digest: row
                    .sha256
                    .as_deref()
                    .filter(|hex| hex.len() == 64)
                    .map(Digest::new_sha256),
            })
        })
        .collect()
}

pub fn summary_from_entry(entry: &AtomEntry) -> Result<ReleaseSummary> {
    let tag = entry.tag().ok_or_else(|| Error::Parse {
        what: "atom entry".to_string(),
        message: format!("no release tag in link {:?}", entry.link),
    })?;
    Ok(ReleaseSummary {
        version: version_from_tag(&tag),
        tag,
        prerelease: false,
        published: entry
            .updated
            .as_deref()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc)),
    })
}

#[async_trait::async_trait]
impl Provider for GitHubWeb {
    async fn latest(&self, repo: &RepoRef, opts: &ResolveOpts) -> Result<Release> {
        let tag = if opts.prerelease {
            let xml = self.get_text(&self.atom_url(repo)).await?;
            let entries = Self::parse_atom(&xml)?;
            let first = entries.first().ok_or_else(|| {
                Error::Provider(format!(
                    "no releases published for {}/{}",
                    repo.owner, repo.repo
                ))
            })?;
            first.tag().ok_or_else(|| Error::Parse {
                what: "atom feed".to_string(),
                message: "first entry has no release tag".to_string(),
            })?
        } else {
            let url = self.latest_url(repo);
            let resp = self
                .client
                .get(&url)
                .header(reqwest::header::USER_AGENT, &self.user_agent)
                .send()
                .await
                .map_err(|source| Error::Http {
                    url: url.clone(),
                    source,
                })?
                .error_for_status()
                .map_err(|source| Error::Http {
                    url: url.clone(),
                    source,
                })?;
            let final_url = resp.url().to_string();
            latest_tag_from_location(&final_url, repo)?
        };
        self.release(repo, &tag).await
    }

    async fn release(&self, repo: &RepoRef, tag: &str) -> Result<Release> {
        let url = self.expanded_assets_url(repo, tag);
        let html = self.get_text(&url).await?;
        let rows = Self::parse_expanded_assets(&html)?;
        let assets = rows_to_assets(&rows, &repo.host)?;
        let published = rows
            .iter()
            .find_map(|r| r.published.as_deref())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc));
        Ok(Release {
            tag: tag.to_string(),
            version: version_from_tag(tag),
            published,
            prerelease: false,
            assets,
        })
    }

    async fn list_releases(&self, repo: &RepoRef, limit: usize) -> Result<Vec<ReleaseSummary>> {
        let xml = self.get_text(&self.atom_url(repo)).await?;
        let entries = Self::parse_atom(&xml)?;
        entries.iter().take(limit).map(summary_from_entry).collect()
    }
}
