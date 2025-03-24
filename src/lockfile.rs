use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

pub struct LockFile {
    path: PathBuf,
}

impl LockFile {
    pub fn try_create(path: PathBuf) -> std::io::Result<Self> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        // file.try_lock()?;
        let pid = std::process::id().to_string().into_bytes();
        file.write_all(&pid)?;
        Ok(Self { path })
    }
}

impl Drop for LockFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
