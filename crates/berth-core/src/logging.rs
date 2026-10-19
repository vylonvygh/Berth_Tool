use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub struct SizeRotatingWriter {
    dir: PathBuf,
    base: OsString,
    max_bytes: u64,
    max_files: usize,
    current: File,
    written: u64,
}

impl SizeRotatingWriter {
    pub fn open(
        dir: &Path,
        base: impl Into<OsString>,
        max_bytes: u64,
        max_files: usize,
    ) -> io::Result<Self> {
        fs::create_dir_all(dir)?;
        let base = base.into();
        let current = OpenOptions::new()
            .create(true)
            .append(true)
            .open(dir.join(&base))?;
        let written = current.metadata()?.len();
        Ok(Self {
            dir: dir.to_path_buf(),
            base,
            max_bytes: max_bytes.max(1),
            max_files: max_files.max(1),
            current,
            written,
        })
    }

    fn rotated_name(&self, index: usize) -> PathBuf {
        let mut name = self.base.clone();
        name.push(format!(".{index}"));
        self.dir.join(name)
    }

    fn rotate(&mut self) -> io::Result<()> {
        let oldest = self.max_files.saturating_sub(1);
        let base_path = self.dir.join(&self.base);

        if oldest == 0 {
            let _ = fs::remove_file(&base_path);
        } else {
            let _ = fs::remove_file(self.rotated_name(oldest));
            for index in (1..oldest).rev() {
                let from = self.rotated_name(index);
                if from.exists() {
                    fs::rename(from, self.rotated_name(index + 1))?;
                }
            }
            if base_path.exists() {
                fs::rename(&base_path, self.rotated_name(1))?;
            }
        }

        self.current = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&base_path)?;
        self.written = 0;
        Ok(())
    }
}

impl Write for SizeRotatingWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.written > 0 && self.written + buf.len() as u64 > self.max_bytes {
            self.rotate()?;
        }
        let written = self.current.write(buf)?;
        self.written += written as u64;
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.current.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn files_in(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn rotates_and_caps_to_max_files() {
        let dir = tempfile::tempdir().unwrap();
        let mut writer = SizeRotatingWriter::open(dir.path(), "berth.log", 16, 3).unwrap();
        for _ in 0..20 {
            writer.write_all(b"0123456789\n").unwrap();
        }
        writer.flush().unwrap();
        let files = files_in(dir.path());
        assert_eq!(files, vec!["berth.log", "berth.log.1", "berth.log.2"]);
        for name in files {
            let size = fs::metadata(dir.path().join(&name)).unwrap().len();
            assert!(size <= 32, "{name} is {size} bytes");
        }
    }

    #[test]
    fn single_file_truncates_in_place() {
        let dir = tempfile::tempdir().unwrap();
        let mut writer = SizeRotatingWriter::open(dir.path(), "berth.log", 8, 1).unwrap();
        for _ in 0..10 {
            writer.write_all(b"abcdefghij").unwrap();
        }
        writer.flush().unwrap();
        assert_eq!(files_in(dir.path()), vec!["berth.log"]);
    }
}
