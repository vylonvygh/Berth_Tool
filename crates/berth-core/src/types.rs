use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderId {
    GitHub,
    GitLab,
    Forgejo,
}

impl fmt::Display for ProviderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProviderId::GitHub => write!(f, "github"),
            ProviderId::GitLab => write!(f, "gitlab"),
            ProviderId::Forgejo => write!(f, "forgejo"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ParseProviderError {
    #[error("unknown provider {0}")]
    Unknown(String),
}

impl FromStr for ProviderId {
    type Err = ParseProviderError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "github" => Ok(ProviderId::GitHub),
            "gitlab" => Ok(ProviderId::GitLab),
            "forgejo" => Ok(ProviderId::Forgejo),
            _ => Err(ParseProviderError::Unknown(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoRef {
    pub provider: ProviderId,
    pub host: String,
    pub owner: String,
    pub repo: String,
}

impl RepoRef {
    pub fn repo_path(&self) -> String {
        format!("{}/{}", self.owner, self.repo)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Version {
    Semver(semver::Version),
    Tag(String),
}

impl Version {
    pub fn as_tag(&self) -> String {
        match self {
            Version::Semver(v) => v.to_string(),
            Version::Tag(t) => t.clone(),
        }
    }

    pub fn strip_v(&self) -> String {
        let s = self.as_tag();
        s.strip_prefix('v')
            .or(s.strip_prefix('V'))
            .unwrap_or(&s)
            .to_string()
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Version::Semver(v) => write!(f, "{v}"),
            Version::Tag(t) => write!(f, "{t}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Digest {
    pub algorithm: String,
    pub hex: String,
}

impl Digest {
    pub fn new_sha256(hex: impl Into<String>) -> Self {
        Self {
            algorithm: "sha256".to_string(),
            hex: hex.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asset {
    pub name: String,
    pub url: Url,
    pub size: Option<u64>,
    pub digest: Option<Digest>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Release {
    pub tag: String,
    pub version: Version,
    pub published: Option<chrono::DateTime<chrono::Utc>>,
    pub prerelease: bool,
    pub assets: Vec<Asset>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseSummary {
    pub tag: String,
    pub version: Version,
    pub prerelease: bool,
    pub published: Option<chrono::DateTime<chrono::Utc>>,
}
