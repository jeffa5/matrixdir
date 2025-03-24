use std::path::PathBuf;

pub fn main() {
    let dir = PathBuf::from("matrixdir");
    let mut matrixdir = matrixdir::matrixdir::MatrixDir::new_writer(dir.clone()).unwrap();
    let pid = std::process::id();
    for i in 0..5 {
        let now = std::time::SystemTime::UNIX_EPOCH
            .elapsed()
            .unwrap()
            .as_millis();
        matrixdir
            .write_event(&format!("{now}: ({pid}) {i}\n"), now)
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    let matrixdir_reader = matrixdir::matrixdir::MatrixDir::new_reader(dir.clone()).unwrap();
    for room in matrixdir_reader.rooms() {
        let room_name = room.name();
        let message_count = room.messages().count();
        let message_files = room.message_files().len();
        println!(
            "found room {:?} with {} messages in {} message files",
            room_name, message_count, message_files
        );
        for message in room.messages() {
            println!("{room_name:?}: {message}");
        }
    }
}
