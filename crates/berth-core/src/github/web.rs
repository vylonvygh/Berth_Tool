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
