use crate::error::{Error, Result};
use crate::provider::{Provider, ResolveOpts};
use crate::types::{Release, ReleaseSummary, RepoRef};

pub struct GitHubWeb {
    _client: reqwest::Client,
    _user_agent: String,
}

impl GitHubWeb {
    pub fn new(user_agent: impl Into<String>) -> Self {
        Self {
            _client: reqwest::Client::builder()
                .use_rustls_tls()
                .build()
                .expect("reqwest client"),
            _user_agent: user_agent.into(),
        }
    }
}

#[async_trait::async_trait]
impl Provider for GitHubWeb {
    async fn latest(&self, repo: &RepoRef, opts: &ResolveOpts) -> Result<Release> {
        let _ = (repo, opts);
        Err(Error::InvalidConfig("not implemented".to_string()))
    }

    async fn release(&self, repo: &RepoRef, tag: &str) -> Result<Release> {
        let _ = (repo, tag);
        Err(Error::InvalidConfig("not implemented".to_string()))
    }

    async fn list_releases(&self, repo: &RepoRef, limit: usize) -> Result<Vec<ReleaseSummary>> {
        let _ = (repo, limit);
        Err(Error::InvalidConfig("not implemented".to_string()))
    }
}
impl GitHubWeb {
    pub fn parse_expanded_assets(html: &str) -> Vec<crate::github::types::AssetRow> {
        let mut rows = Vec::new();
        let doc = scraper::Html::parse_document(html);
        let name_selector = scraper::Selector::parse("span.text-bold").unwrap();
        let link_selector = scraper::Selector::parse("a[href*='/releases/download/']").unwrap();
        // collect name spans
        let mut names: Vec<String> = Vec::new();
        for el in doc.select(&name_selector) {
            let text = el.text().collect::<Vec<_>>().join(" ");
            names.push(text);
        }
        // collect hrefs
        let mut hrefs: Vec<String> = Vec::new();
        for el in doc.select(&link_selector) {
            if let Some(h) = el.value().attr("href") {
                hrefs.push(h.to_string());
            }
        }
        // sizes and digests appear later; simple heuristic: scan text for patterns
        let text = doc.root_element().text().collect::<Vec<_>>().join("\n");
        let mut sizes: Vec<Option<u64>> = vec![None; names.len().max(hrefs.len())];
        let re_sz = regex::Regex::new(r"(\d+(?:\.\d+)?)\s*(B|KB|MB|GB|TB)").unwrap();
        for cap in re_sz.captures_iter(&text) {
            let num: f64 = cap[1].parse().unwrap_or(0.0);
            let mul = match &cap[2] {
                "B" => 1.0,
                "KB" => 1024.0,
                "MB" => 1024.0 * 1024.0,
                "GB" => 1024.0 * 1024.0 * 1024.0,
                "TB" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
                _ => 1.0,
            };
            let sz = Some((num * mul) as u64);
            // assign to the next unassigned or best match
            for (i, _) in names.iter().enumerate().take(sizes.len()) {
                if sizes[i].is_none() {
                    // prefer if near name
                    sizes[i] = sz;
                    break;
                }
            }
        }
        let mut digests: Vec<Option<String>> = vec![None; names.len().max(hrefs.len())];
        let re_sha = regex::Regex::new(r"sha256:([a-f0-9]{64})").unwrap();
        for (i, name) in names.iter().enumerate().take(digests.len()) {
            if let Some(idx) = text.find(name) {
                let window = &text[idx..(idx + 400).min(text.len())];
                if let Some(cap) = re_sha.captures(window) {
                    digests[i] = Some(cap[1].to_string());
                }
            }
        }
        let n = names.len().max(hrefs.len());
        for i in 0..n {
            rows.push(crate::github::types::AssetRow {
                name: names.get(i).cloned().unwrap_or_default(),
                url_relative: hrefs.get(i).cloned().unwrap_or_default(),
                size: sizes.get(i).cloned().flatten(),
                sha256: digests.get(i).cloned().flatten(),
            });
        }
        rows
    }
}
