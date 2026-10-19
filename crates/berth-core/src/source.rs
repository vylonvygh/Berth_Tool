use std::str::FromStr;

use regex::Regex;
use url::Url;

use crate::error::{Error, Result};
use crate::types::{ProviderId, RepoRef};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSpec {
    pub repo: RepoRef,
    pub version: Option<String>,
}

impl SourceSpec {
    pub fn from_input(input: &str) -> Result<Self> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(Error::InvalidConfig("empty source".to_string()));
        }

        let mut candidates = Vec::new();
        if let Some(idx) = trimmed.rfind('@') {
            let prefix = &trimmed[..idx];
            let ver = &trimmed[idx + 1..];
            if !ver.is_empty() && !prefix.contains("://") {
                candidates.push((prefix, Some(ver.to_string())));
            }
        }
        candidates.push((trimmed, None));

        for (rest, version) in candidates {
            let rest = rest.trim();
            if let Some(spec) = try_parse(rest)? {
                return Ok(SourceSpec {
                    repo: spec,
                    version,
                });
            }
        }
        Err(Error::InvalidConfig(format!(
            "cannot parse source: {input}"
        )))
    }
}

fn try_parse(s: &str) -> Result<Option<RepoRef>> {
    if let Some(spec) = try_url(s)? {
        return Ok(Some(spec));
    }
    if let Some(spec) = try_explicit_prefix(s)? {
        return Ok(Some(spec));
    }
    if let Some(spec) = try_hostpath(s)? {
        return Ok(Some(spec));
    }
    if let Some(spec) = try_shorthand(s)? {
        return Ok(Some(spec));
    }
    Ok(None)
}

fn try_url(s: &str) -> Result<Option<RepoRef>> {
    if !s.contains("://") {
        return Ok(None);
    }
    let url =
        Url::parse(s).map_err(|_| Error::InvalidConfig(format!("cannot parse source: {s}")))?;
    let host = url
        .host_str()
        .ok_or_else(|| Error::InvalidConfig(format!("missing host in URL: {s}")))?;
    let host = host.to_string();
    let path = url.path();
    let stripped = path.strip_prefix('/').unwrap_or(path);
    let parts: Vec<_> = stripped.split('/').filter(|x| !x.is_empty()).collect();
    if parts.is_empty() {
        return Ok(None);
    }
    let owner = parts[0].to_string();
    let repo = parts.get(1).cloned().unwrap_or_default();
    if parts.len() >= 4 && parts[1] == "releases" && parts[2] == "tag" {
        // release/tag URL, owner/repo are parts[0],parts[1] already
    }
    let repo = repo
        .strip_suffix(".git")
        .map(|x| x.to_string())
        .unwrap_or_else(|| repo.to_string());
    let provider = detect_provider(&host);
    if owner.is_empty() || repo.is_empty() {
        return Ok(None);
    }
    Ok(Some(RepoRef {
        provider,
        host,
        owner,
        repo,
    }))
}

fn try_explicit_prefix(s: &str) -> Result<Option<RepoRef>> {
    let re = Regex::new(r"^(?P<prov>github|gitlab|forgejo):(?P<owner>[^/]+)/(?P<repo>[^/\s@]+)$")
        .unwrap();
    if let Some(caps) = re.captures(s) {
        let provider = ProviderId::from_str(&caps["prov"])
            .map_err(|_| Error::InvalidConfig(format!("unsupported provider in: {s}")))?;
        let host = default_host(provider);
        return Ok(Some(RepoRef {
            provider,
            host,
            owner: caps["owner"].to_string(),
            repo: caps["repo"].to_string(),
        }));
    }
    Ok(None)
}

fn try_hostpath(s: &str) -> Result<Option<RepoRef>> {
    let re = Regex::new(r"^(?P<host>[^/]+)\.com/(?P<owner>[^/]+)/(?P<repo>[^/\s@]+)(?:\.git)?$")
        .unwrap();
    if let Some(caps) = re.captures(s) {
        let host_full = format!("{}.com", &caps["host"]);
        let provider = detect_provider(&host_full);
        return Ok(Some(RepoRef {
            provider,
            host: host_full,
            owner: caps["owner"].to_string(),
            repo: caps["repo"].to_string(),
        }));
    }
    Ok(None)
}

fn try_shorthand(s: &str) -> Result<Option<RepoRef>> {
    let re = Regex::new(r"^(?P<owner>[^/\s@]+)\/(?P<repo>[^/\s@]+)$").unwrap();
    if let Some(caps) = re.captures(s) {
        let provider = ProviderId::GitHub;
        let host = default_host(provider);
        return Ok(Some(RepoRef {
            provider,
            host,
            owner: caps["owner"].to_string(),
            repo: caps["repo"].to_string(),
        }));
    }
    Ok(None)
}

fn detect_provider(host: &str) -> ProviderId {
    let h = host.to_ascii_lowercase();
    match h.as_str() {
        "github.com" | "www.github.com" => ProviderId::GitHub,
        "gitlab.com" | "www.gitlab.com" => ProviderId::GitLab,
        _ => {
            if h.ends_with(".github.com") {
                ProviderId::GitHub
            } else if h.ends_with(".gitlab.com") {
                ProviderId::GitLab
            } else if h.contains("forgejo") {
                ProviderId::Forgejo
            } else if h.contains("gitlab") {
                ProviderId::GitLab
            } else {
                ProviderId::GitHub
            }
        }
    }
}

fn default_host(provider: ProviderId) -> String {
    match provider {
        ProviderId::GitHub => "github.com".to_string(),
        ProviderId::GitLab => "gitlab.com".to_string(),
        ProviderId::Forgejo => "forgejo.com".to_string(),
    }
}
