use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not determine the user's home directory")]
    NoHomeDir,

    #[error("could not determine the XDG project directories")]
    NoProjectDirs,

    #[error("could not determine the XDG state directory")]
    NoStateDir,

    #[error("failed to create directory {path}: {source}")]
    CreateDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to read config {path}: {source}")]
    ReadConfig {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("invalid config {path}: {message}")]
    ParseConfig { path: PathBuf, message: String },

    #[error("invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("HTTP request to {url} failed: {source}")]
    Http {
        url: String,
        #[source]
        source: reqwest::Error,
    },

    #[error("failed to parse {what}: {message}")]
    Parse { what: String, message: String },

    #[error("provider error: {0}")]
    Provider(String),

    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("corrupt berth state: {0}")]
    CorruptState(String),

    #[error("database schema version {found} is newer than this build supports ({supported})")]
    SchemaTooNew { found: u32, supported: u32 },

    #[error("another berth process already holds the lock ({path})")]
    Locked { path: PathBuf },

    #[error("failed to lock {path}: {source}")]
    Lock {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
