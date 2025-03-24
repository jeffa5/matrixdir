use std::path::PathBuf;

use notify::Watcher as _;

pub fn main() {
    let dir = PathBuf::from("matrixdir");

    let mut watcher = notify::recommended_watcher(matrixdir::watcher::MatrixDirWatcher::new(
        dir.clone(),
        |event| eprintln!("{:?}", event),
    ))
    .unwrap();
    watcher
        .watch(&dir, notify::RecursiveMode::Recursive)
        .unwrap();

    let mut matrixdir = matrixdir::matrixdir::MatrixDir::new_writer(dir.clone()).unwrap();
    let pid = std::process::id();
    let room_name = "default2".to_owned();
    for i in 0..5 {
        let now = std::time::SystemTime::UNIX_EPOCH
            .elapsed()
            .unwrap()
            .as_millis();
        matrixdir
            .write_event(&format!("{now}: ({pid}) {i}\n"), room_name.clone(), now)
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    let matrixdir_reader = matrixdir::matrixdir::MatrixDir::new_reader(dir.clone()).unwrap();
    for room in matrixdir_reader.rooms() {
        let room_name = room.name();
        let message_count = room.messages(false).count();
        let message_files = room.message_files().len();
        println!(
            "found room {:?} with {} messages in {} message files",
            room_name, message_count, message_files
        );
        // for message in room.messages() {
        //     println!("{room_name:?}: {message}");
        // }
    }
}
