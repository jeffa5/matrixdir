use crate::lockfile::LockFile;
use crate::matrixroomdir::MatrixRoomDir;
use crate::read_write::ReaderWriter;
use std::collections::BTreeMap;
use std::path::PathBuf;

pub struct MatrixDir<RW> {
    path: PathBuf,
    // optional as we don't need it to read
    _lockfile: Option<LockFile>,
    rooms: BTreeMap<String, MatrixRoomDir<RW>>,
}

impl MatrixDir<crate::read_write::Write> {
    pub fn new_writer(path: PathBuf) -> std::io::Result<Self> {
        if !path.exists() {
            std::fs::create_dir(&path)?;
        }

        let lockfile = LockFile::try_create(path.join("lock.pid"))?;
        let mut rooms = BTreeMap::new();
        for entry in path.read_dir()? {
            let entry = entry?;
            if entry.path().is_dir() {
                let room = MatrixRoomDir::new_writer(entry.path())?;
                let room_name = room.name();
                rooms.insert(room_name, room);
            }
        }
        Ok(Self {
            path,
            _lockfile: Some(lockfile),
            rooms,
        })
    }

    pub fn write_event(&mut self, event: &str, timestamp: u128) -> std::io::Result<()> {
        let room_name = "default".to_owned();
        // TODO: extract room from event and use that for rooms lookup
        let room = self
            .rooms
            .entry(room_name.clone())
            .or_insert_with(|| MatrixRoomDir::new_writer(self.path.join(room_name)).unwrap());
        room.write_event(event, timestamp)
    }
}

impl MatrixDir<crate::read_write::Read> {
    pub fn new_reader(path: PathBuf) -> std::io::Result<Self> {
        if !path.exists() {
            std::fs::create_dir(&path)?;
        }

        let mut rooms = BTreeMap::new();
        for entry in path.read_dir()? {
            let entry = entry?;
            if entry.path().is_dir() {
                let room = MatrixRoomDir::new_reader(entry.path())?;
                let room_name = room.name();
                rooms.insert(room_name, room);
            }
        }
        Ok(Self {
            path,
            _lockfile: None,
            rooms,
        })
    }
}

impl<RW: ReaderWriter> MatrixDir<RW> {
    pub fn rooms(&self) -> Vec<&MatrixRoomDir<RW>> {
        self.rooms.values().collect()
    }
}
