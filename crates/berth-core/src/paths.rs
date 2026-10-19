use std::path::{Path, PathBuf};

use directories::{BaseDirs, ProjectDirs};

use crate::error::{Error, Result};

pub const APP_NAME: &str = "berth";

#[derive(Debug, Clone)]
pub struct Paths {
    config_dir: PathBuf,
    data_dir: PathBuf,
    state_dir: PathBuf,
    cache_dir: PathBuf,
    runtime_dir: Option<PathBuf>,
    home_dir: PathBuf,
}

impl Paths {
    pub fn new(
        config_dir: PathBuf,
        data_dir: PathBuf,
        state_dir: PathBuf,
        cache_dir: PathBuf,
        runtime_dir: Option<PathBuf>,
        home_dir: PathBuf,
    ) -> Self {
        Self {
            config_dir,
            data_dir,
            state_dir,
            cache_dir,
            runtime_dir,
            home_dir,
        }
    }

    pub fn discover() -> Result<Self> {
        let project = ProjectDirs::from("", "", APP_NAME).ok_or(Error::NoProjectDirs)?;
        let base = BaseDirs::new().ok_or(Error::NoHomeDir)?;
        let runtime_dir = std::env::var_os("XDG_RUNTIME_DIR")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from);
        Ok(Self {
            config_dir: project.config_dir().to_path_buf(),
            data_dir: project.data_dir().to_path_buf(),
            state_dir: project
                .state_dir()
                .map(Path::to_path_buf)
                .ok_or(Error::NoStateDir)?,
            cache_dir: project.cache_dir().to_path_buf(),
            runtime_dir,
            home_dir: base.home_dir().to_path_buf(),
        })
    }

    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub fn state_dir(&self) -> &Path {
        &self.state_dir
    }

    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    pub fn home_dir(&self) -> &Path {
        &self.home_dir
    }

    pub fn config_file(&self) -> PathBuf {
        self.config_dir.join("berth.toml")
    }

    pub fn db_file(&self) -> PathBuf {
        self.data_dir.join("berth.db")
    }

    pub fn store_dir(&self) -> PathBuf {
        self.data_dir.join("store")
    }

    pub fn log_file(&self) -> PathBuf {
        self.state_dir.join("berth.log")
    }

    pub fn tmp_dir(&self) -> PathBuf {
        self.cache_dir.join("tmp")
    }

    pub fn downloads_dir(&self) -> PathBuf {
        self.cache_dir.join("downloads")
    }

    pub fn lock_file(&self) -> PathBuf {
        match &self.runtime_dir {
            Some(dir) if dir.is_dir() => dir.join("berth.lock"),
            _ => self.data_dir.join("berth.lock"),
        }
    }

    pub fn default_bin_dir(&self) -> PathBuf {
        self.home_dir.join(".local/bin")
    }

    pub fn ensure_dirs(&self) -> Result<()> {
        for dir in [
            self.data_dir.clone(),
            self.state_dir.clone(),
            self.cache_dir.clone(),
            self.store_dir(),
            self.tmp_dir(),
            self.downloads_dir(),
        ] {
            std::fs::create_dir_all(&dir)
                .map_err(|source| Error::CreateDir { path: dir, source })?;
        }
        tracing::debug!(data = %self.data_dir.display(), "ensured berth directories");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(runtime: Option<&str>) -> Paths {
        Paths::new(
            PathBuf::from("/cfg/berth"),
            PathBuf::from("/data/berth"),
            PathBuf::from("/state/berth"),
            PathBuf::from("/cache/berth"),
            runtime.map(PathBuf::from),
            PathBuf::from("/home/alice"),
        )
    }

    #[test]
    fn derives_config_data_state_cache_files() {
        let paths = sample(None);
        assert_eq!(paths.config_file(), PathBuf::from("/cfg/berth/berth.toml"));
        assert_eq!(paths.db_file(), PathBuf::from("/data/berth/berth.db"));
        assert_eq!(paths.store_dir(), PathBuf::from("/data/berth/store"));
        assert_eq!(paths.log_file(), PathBuf::from("/state/berth/berth.log"));
        assert_eq!(paths.tmp_dir(), PathBuf::from("/cache/berth/tmp"));
        assert_eq!(
            paths.downloads_dir(),
            PathBuf::from("/cache/berth/downloads")
        );
    }

    #[test]
    fn default_bin_dir_is_local_bin() {
        assert_eq!(
            sample(None).default_bin_dir(),
            PathBuf::from("/home/alice/.local/bin")
        );
    }

    #[test]
    fn lock_file_prefers_existing_runtime_dir() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = dir.path().join("run");
        std::fs::create_dir_all(&runtime).unwrap();
        let paths = sample(Some(runtime.to_str().unwrap()));
        assert_eq!(paths.lock_file(), runtime.join("berth.lock"));
    }

    #[test]
    fn lock_file_falls_back_when_runtime_missing() {
        let paths = sample(Some("/nonexistent/run/user/1000"));
        assert_eq!(paths.lock_file(), PathBuf::from("/data/berth/berth.lock"));
    }
}
