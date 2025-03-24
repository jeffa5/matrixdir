use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::marker::PhantomData;
use std::path::PathBuf;

use crate::read_write::ReaderWriter;

pub struct MatrixFile<RW> {
    path: PathBuf,
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
