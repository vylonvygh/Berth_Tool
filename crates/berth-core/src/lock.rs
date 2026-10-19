use std::fs::{File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

use fd_lock::RwLock;

use crate::error::{Error, Result};

pub struct ProcessLock {
    path: PathBuf,
    _guard: fd_lock::RwLockWriteGuard<'static, File>,
}

impl ProcessLock {
    pub fn acquire(path: &Path) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)
            .map_err(|source| Error::Lock {
                path: path.to_path_buf(),
                source,
            })?;

        let lock: &'static mut RwLock<File> = Box::leak(Box::new(RwLock::new(file)));
        match lock.try_write() {
            Ok(guard) => {
                tracing::debug!(path = %path.display(), "acquired berth process lock");
                Ok(Self {
                    path: path.to_path_buf(),
                    _guard: guard,
                })
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => Err(Error::Locked {
                path: path.to_path_buf(),
            }),
            Err(source) => Err(Error::Lock {
                path: path.to_path_buf(),
                source,
            }),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn second_acquire_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("berth.lock");
        let first = ProcessLock::acquire(&path).unwrap();
        let second = ProcessLock::acquire(&path);
        assert!(matches!(second, Err(Error::Locked { .. })));
        assert_eq!(first.path(), path.as_path());
        drop(first);
    }

    #[test]
    fn lock_is_released_on_drop() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("berth.lock");
        drop(ProcessLock::acquire(&path).unwrap());
        assert!(ProcessLock::acquire(&path).is_ok());
    }
}
