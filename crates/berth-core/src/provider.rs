use crate::error::Result;
use crate::types::{Release, ReleaseSummary, RepoRef};

#[async_trait::async_trait]
pub trait Provider {
    async fn latest(&self, repo: &RepoRef, opts: &ResolveOpts) -> Result<Release>;
    async fn release(&self, repo: &RepoRef, tag: &str) -> Result<Release>;
    async fn list_releases(&self, repo: &RepoRef, limit: usize) -> Result<Vec<ReleaseSummary>>;
}

#[derive(Debug, Clone, Default)]
pub struct ResolveOpts {
    pub prerelease: bool,
}
