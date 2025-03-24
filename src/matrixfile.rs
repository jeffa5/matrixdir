use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::marker::PhantomData;
use std::path::Path;
use std::path::PathBuf;

use crate::read_write::ReaderWriter;

#[derive(Debug)]
pub struct MatrixFile<RW> {
    pub(crate) path: PathBuf,
    file: File,
    _rw: PhantomData<RW>,
}

impl MatrixFile<crate::read_write::Read> {
    pub fn new_reader(path: PathBuf) -> std::io::Result<Self> {
        let file = OpenOptions::new().read(true).open(&path)?;
        Ok(Self {
            path,
            file,
            _rw: PhantomData::default(),
        })
    }

    fn open_file(path: &Path) -> std::io::Result<File> {
        OpenOptions::new().read(true).open(&path)
    }

    pub fn messages(&self, follow: bool) -> MessageIterator {
        let file = Self::open_file(&self.path).unwrap();
        MessageIterator {
            bufreader: BufReader::new(file),
            closed: false,
            follow,
        }
    }
}

impl MatrixFile<crate::read_write::Write> {
    pub fn new_writer(path: PathBuf) -> std::io::Result<Self> {
        let file = OpenOptions::new().append(true).create(true).open(&path)?;
        Ok(Self {
            path,
            file,
            _rw: PhantomData::default(),
        })
    }
}

impl<RW: ReaderWriter> MatrixFile<RW> {
    pub fn size(&self) -> u64 {
        self.file.metadata().map_or(0, |m| m.len())
    }
}

impl Write for MatrixFile<crate::read_write::Write> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.file.write(bytes)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}

#[derive(Debug)]
pub struct MessageIterator {
    bufreader: BufReader<File>,
    closed: bool,
    follow: bool,
}

impl Iterator for MessageIterator {
    type Item = Option<String>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.closed {
            return None;
        }

        let mut buf = Vec::new();
        let read_result = self.bufreader.read_until(b'\n', &mut buf);
        match read_result {
            Ok(0) => {
                if self.follow {
                    Some(None)
                } else {
                    self.closed = true;
                    None
                }
            }
            Ok(_) => {
                // remove the '\n'
                buf.pop();
                let line = String::from_utf8(buf).unwrap();
                return Some(Some(line));
            }
            Err(_) => {
                self.closed = true;
                None
            }
        }
    }
}
