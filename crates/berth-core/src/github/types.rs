#[derive(Debug, Clone)]
pub struct AssetRow {
    pub name: String,
    pub url_relative: String,
    pub size: Option<u64>,
    pub sha256: Option<String>,
    pub published: Option<String>,
}
