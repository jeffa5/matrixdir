use std::{collections::BTreeMap, path::PathBuf};

use notify::{EventHandler, EventKind};

use crate::{matrixdir::MatrixDir, matrixfile::FileMessageIterator, MatrixFile, MatrixRoomDir};

pub trait MatrixEventHandler {
    fn handle(&mut self, event: String);
}

impl<F> MatrixEventHandler for F
where
    F: FnMut(String),
{
    fn handle(&mut self, event: String) {
        self(event)
    }
}

pub struct MatrixDirWatcher<H> {
    root: PathBuf,
    file_followers: BTreeMap<PathBuf, FileMessageIterator>,
    event_handler: H,
}

impl<H: MatrixEventHandler> MatrixDirWatcher<H> {
    pub fn new(root: PathBuf, mut event_handler: H) -> Self {
        let abs_root = root.canonicalize().unwrap();
        let matrixdir = MatrixDir::new_reader(root).unwrap();
        let mut file_followers = BTreeMap::new();
        for room in matrixdir.rooms() {
            for file in room.message_files() {
                let mut follower = file.messages(true);
                emit_events_from_follower(&mut follower, &mut event_handler);
                let path = file.path.canonicalize().unwrap();
                file_followers.insert(path, follower);
            }
        }
        Self {
            root: abs_root,
            file_followers,
            event_handler,
        }
    }

    pub fn handle_create_file(&mut self, path: PathBuf) {
        let rel_path = path.strip_prefix(&self.root).unwrap();
        let parts: Vec<_> = rel_path.iter().collect();
        if parts.len() == 2 {
            if self.file_followers.contains_key(&path) {
                eprintln!("Skipping adding file that we are already tracking: {path:?}");
            } else {
                // eprintln!("Adding file follower for new file: {path:?}");
                let matrixfile = MatrixFile::new_reader(path.clone()).unwrap();
                let mut follower = matrixfile.messages(true);
                emit_events_from_follower(&mut follower, &mut self.event_handler);
                self.file_followers.insert(path.clone(), follower);
            }
        }
    }

    pub fn handle_create_folder(&mut self, path: PathBuf) {
        let rel_path = path.strip_prefix(&self.root).unwrap();
        if rel_path.iter().count() == 1 {
            // eprintln!("Got new room: {path:?}");
            let room = MatrixRoomDir::new_reader(path).unwrap();
            for file in room.message_files() {
                let mut follower = file.messages(true);
                emit_events_from_follower(&mut follower, &mut self.event_handler);
                let path = file.path.canonicalize().unwrap();
                self.file_followers.insert(path, follower);
            }
        } else {
            eprintln!("Folder created but not at root of matrixdir: {path:?}");
        }
    }

    pub fn handle_modify(&mut self, path: PathBuf) {
        if let Some(follower) = self.file_followers.get_mut(&path) {
            // eprintln!("Got modify event on followed file, getting new events: {path:?}");
            emit_events_from_follower(follower, &mut self.event_handler);
        } else {
            eprintln!("Got modify event for file we aren't following: {path:?}");
        }
    }
}

pub fn emit_events_from_follower<H: MatrixEventHandler>(
    follower: &mut FileMessageIterator,
    handler: &mut H,
) {
    while let Some(Some(event)) = follower.next() {
        handler.handle(event);
    }
}

impl<H: MatrixEventHandler + Send + 'static> EventHandler for MatrixDirWatcher<H> {
    fn handle_event(&mut self, event: notify::Result<notify::Event>) {
        let Ok(event) = event else { return };
        // TODO: check for rescan?

        // have some map of the matrixfiles that we are watching, use create and remove events
        // to manage these
        // modify events then lead to reading more on their iterators, may need to change the
        // matrixfile iterator to not end when it reaches the end of the file
        if event.paths.len() == 1 && event.paths[0] == self.root.join("lock.pid") {
            return;
        }

        match event.kind {
            EventKind::Create(notify::event::CreateKind::File) => {
                for path in event.paths {
                    self.handle_create_file(path);
                }
            }
            EventKind::Create(notify::event::CreateKind::Folder) => {
                for path in event.paths {
                    self.handle_create_folder(path);
                }
            }
            EventKind::Remove(notify::event::RemoveKind::File) => {
                unimplemented!()
            }
            EventKind::Remove(notify::event::RemoveKind::Folder) => {
                unimplemented!()
            }
            EventKind::Modify(notify::event::ModifyKind::Data(_)) => {
                for path in event.paths {
                    self.handle_modify(path);
                }
            }
            _ => {
                // skip
            }
        }
    }
}
