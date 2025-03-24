use crate::matrixfile::MatrixFile;
use std::collections::BTreeMap;
use std::io::Write;
use std::path::PathBuf;

pub struct MatrixRoomDir<RW> {
    path: PathBuf,
    files: BTreeMap<u128, MatrixFile<RW>>,
}

impl MatrixRoomDir<crate::read_write::Write> {
    pub fn new_writer(path: PathBuf) -> std::io::Result<Self> {
        if !path.exists() {
            std::fs::create_dir(&path)?;
        }

        let mut files = BTreeMap::new();
        for entry in path.read_dir()? {
            let entry = entry?;
            if entry.path().is_file() {
                let file_name = entry
                    .path()
                    .file_stem()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned();
                let filename_timestamp: u128 = file_name.parse().unwrap();
                let matrix_file = MatrixFile::new_writer(entry.path())?;
                files.insert(filename_timestamp, matrix_file);
            }
        }
        Ok(Self {
            path,
            files,
        })
    }

    pub fn write_event(&mut self, event: &str, timestamp: u128) -> std::io::Result<()> {
        let file_timestamp = self.files.range(..=timestamp).last().map(|(ts, _)| *ts);
        let file = if let Some(file_timestamp) = file_timestamp {
            self.files.get_mut(&file_timestamp).unwrap()
        } else {
            let path = self.path.join(format!("{timestamp}.jsonl"));
            let mf = MatrixFile::new_writer(path)?;
            self.files.insert(timestamp, mf);
            self.files.get_mut(&timestamp).unwrap()
        };
        file.write_all(event.as_bytes())
    }
}
