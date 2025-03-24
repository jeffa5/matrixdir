use std::collections::VecDeque;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read as _;
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

    pub fn messages(&self) -> MessageIterator {
        MessageIterator {
            file: Self::open_file(&self.path).unwrap(),
            closed: false,
            big_buf: VecDeque::new(),
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
    file: File,
    closed: bool,
    big_buf: VecDeque<u8>,
}

impl Iterator for MessageIterator {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.closed {
            return None;
        }

        let mut buf = vec![0; 1024];
        loop {
            let read = self.file.read(&mut buf);
            if read.is_err() {
                self.closed = true;
                return None;
            }

            let read = read.unwrap();

            self.big_buf.extend(&buf[..read]);
            if let Some(pos) = self.big_buf.iter().position(|b| *b == b'\n') {
                let rest = self.big_buf.split_off(pos);
                let line = String::from_utf8(self.big_buf.make_contiguous().to_vec()).unwrap();
                self.big_buf = rest;
                // remove the newline
                self.big_buf.pop_front();
                return Some(line);
            }

            if read == 0 && self.big_buf.is_empty() {
                self.closed = true;
                return None;
            }
        }
    }
}
