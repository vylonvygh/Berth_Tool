use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::{Error, Result};

pub const KEEP_VERSIONS_DEFAULT: u32 = 2;
pub const CONCURRENCY_DEFAULT: u32 = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorChoice {
    #[default]
    Auto,
    Always,
    Never,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    #[default]
    Web,
    Api,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigFile {
    pub bin_dir: Option<PathBuf>,
    pub keep_versions: Option<u32>,
    pub concurrency: Option<u32>,
    pub color: Option<ColorChoice>,
    #[serde(default)]
    pub defaults: DefaultsFile,
    #[serde(default)]
    pub hosts: BTreeMap<String, HostFile>,
    #[serde(default)]
    pub cache: CacheFile,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DefaultsFile {
    pub prerelease: Option<bool>,
    pub mode: Option<Mode>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostFile {
    pub mode: Option<Mode>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CacheFile {
    pub keep_downloads: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostConfig {
    pub mode: Mode,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub bin_dir: PathBuf,
    pub keep_versions: u32,
    pub concurrency: u32,
    pub color: ColorChoice,
    pub prerelease: bool,
    pub mode: Mode,
    pub keep_downloads: bool,
    pub hosts: BTreeMap<String, HostConfig>,
}

impl Config {
    pub fn defaults(default_bin_dir: PathBuf) -> Self {
        Self {
            bin_dir: default_bin_dir,
            keep_versions: KEEP_VERSIONS_DEFAULT,
            concurrency: CONCURRENCY_DEFAULT,
            color: ColorChoice::default(),
            prerelease: false,
            mode: Mode::default(),
            keep_downloads: false,
            hosts: BTreeMap::new(),
        }
    }

    pub fn load(path: &Path, default_bin_dir: PathBuf) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::defaults(default_bin_dir));
        }
        let text = std::fs::read_to_string(path).map_err(|source| Error::ReadConfig {
            path: path.to_path_buf(),
            source,
        })?;
        let file: ConfigFile = toml::from_str(&text).map_err(|error| Error::ParseConfig {
            path: path.to_path_buf(),
            message: error.to_string(),
        })?;
        let config = Self::resolve(file, default_bin_dir)?;
        tracing::debug!(path = %path.display(), "loaded configuration");
        Ok(config)
    }

    pub fn resolve(file: ConfigFile, default_bin_dir: PathBuf) -> Result<Self> {
        let mut config = Self::defaults(default_bin_dir);

        if let Some(bin_dir) = file.bin_dir {
            config.bin_dir = bin_dir;
        }
        if let Some(keep_versions) = file.keep_versions {
            if keep_versions == 0 {
                return Err(Error::InvalidConfig(
                    "keep_versions must be at least 1".to_string(),
                ));
            }
            config.keep_versions = keep_versions;
        }
        if let Some(concurrency) = file.concurrency {
            if concurrency == 0 {
                return Err(Error::InvalidConfig(
                    "concurrency must be at least 1".to_string(),
                ));
            }
            config.concurrency = concurrency;
        }
        if let Some(color) = file.color {
            config.color = color;
        }
        if let Some(prerelease) = file.defaults.prerelease {
            config.prerelease = prerelease;
        }
        if let Some(mode) = file.defaults.mode {
            config.mode = mode;
        }
        if let Some(keep_downloads) = file.cache.keep_downloads {
            config.keep_downloads = keep_downloads;
        }
        config.hosts = file
            .hosts
            .into_iter()
            .map(|(host, entry)| {
                (
                    host,
                    HostConfig {
                        mode: entry.mode.unwrap_or_default(),
                    },
                )
            })
            .collect();

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bin_dir() -> PathBuf {
        PathBuf::from("/home/alice/.local/bin")
    }

    #[test]
    fn defaults_when_no_file() {
        let config = Config::defaults(bin_dir());
        assert_eq!(config.keep_versions, 2);
        assert_eq!(config.concurrency, 6);
        assert_eq!(config.color, ColorChoice::Auto);
        assert!(!config.prerelease);
        assert_eq!(config.mode, Mode::Web);
        assert!(!config.keep_downloads);
        assert!(config.hosts.is_empty());
    }

    #[test]
    fn parses_spec_example() {
        let text = r#"
bin_dir = "/home/alice/bin"
keep_versions = 3
concurrency = 4
color = "never"

[defaults]
prerelease = true
mode = "api"

[hosts."github.com"]
mode = "web"

[hosts."git.example.com"]
mode = "api"

[cache]
keep_downloads = true
"#;
        let file: ConfigFile = toml::from_str(text).unwrap();
        let config = Config::resolve(file, bin_dir()).unwrap();
        assert_eq!(config.bin_dir, PathBuf::from("/home/alice/bin"));
        assert_eq!(config.keep_versions, 3);
        assert_eq!(config.concurrency, 4);
        assert_eq!(config.color, ColorChoice::Never);
        assert!(config.prerelease);
        assert_eq!(config.mode, Mode::Api);
        assert!(config.keep_downloads);
        assert_eq!(
            config.hosts.get("github.com"),
            Some(&HostConfig { mode: Mode::Web })
        );
        assert_eq!(
            config.hosts.get("git.example.com"),
            Some(&HostConfig { mode: Mode::Api })
        );
    }

    #[test]
    fn missing_file_uses_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let config = Config::load(&dir.path().join("berth.toml"), bin_dir()).unwrap();
        assert_eq!(config.keep_versions, 2);
    }

    #[test]
    fn empty_file_uses_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("berth.toml");
        std::fs::write(&path, "").unwrap();
        let config = Config::load(&path, bin_dir()).unwrap();
        assert_eq!(config.bin_dir, bin_dir());
    }

    #[test]
    fn unknown_key_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("berth.toml");
        std::fs::write(&path, "keep_versionz = 3\n").unwrap();
        let error = Config::load(&path, bin_dir()).unwrap_err();
        assert!(matches!(error, Error::ParseConfig { .. }));
    }

    #[test]
    fn zero_keep_versions_is_rejected() {
        let file: ConfigFile = toml::from_str("keep_versions = 0\n").unwrap();
        let error = Config::resolve(file, bin_dir()).unwrap_err();
        assert!(matches!(error, Error::InvalidConfig(_)));
    }
}
