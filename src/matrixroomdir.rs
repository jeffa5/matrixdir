use crate::matrixfile::MatrixFile;
use crate::read_write::ReaderWriter;
use std::collections::BTreeMap;
use std::io::Write;
use std::ops::Bound;
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
        Ok(Self { path, files })
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

impl MatrixRoomDir<crate::read_write::Read> {
    pub fn new_reader(path: PathBuf) -> std::io::Result<Self> {
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
                let matrix_file = MatrixFile::new_reader(entry.path())?;
                files.insert(filename_timestamp, matrix_file);
            }
        }
        Ok(Self { path, files })
    }
}

impl<RW: ReaderWriter> MatrixRoomDir<RW> {
    pub fn name(&self) -> String {
        self.path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned()
    }

    pub fn messages(&self) -> MessageIterator {
        // iterate over each events file yielding the messages line by line
        MessageIterator {
            files: self
                .files
                .iter()
                .map(|(k, v)| (*k, MatrixFile::new_reader(v.path.clone()).unwrap()))
                .collect(),
            current_file: None,
            closed: false,
        }
    }

    pub fn message_files(&self) -> Vec<&MatrixFile<RW>> {
        self.files.values().collect()
    }
}

#[derive(Debug)]
pub struct MessageIterator {
    files: BTreeMap<u128, MatrixFile<crate::read_write::Read>>,
    current_file: Option<(u128, crate::matrixfile::MessageIterator)>,
    closed: bool,
}

impl Iterator for MessageIterator {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.closed {
            return None;
        }

        if let Some((current_ts, current_iter)) = &mut self.current_file {
            if let Some(msg) = current_iter.next() {
                Some(msg)
            } else {
                let next_file = self
                    .files
                    .range((Bound::Excluded(*current_ts), Bound::Unbounded))
                    .next();
                if let Some(next_file) = next_file {
                    self.current_file = Some((*next_file.0, next_file.1.messages()));
                    self.next()
                } else {
                    self.current_file = None;
                    self.closed = true;
                    None
                }
            }
        } else {
            let next_file = self.files.iter().next();
            if let Some(next_file) = next_file {
                self.current_file = Some((*next_file.0, next_file.1.messages()));
                self.next()
            } else {
                None
            }
        }
    }
}
