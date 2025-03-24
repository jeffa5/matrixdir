pub fn main() {
    let mut matrixdir = matrixdir::matrixdir::MatrixDir::new_writer("matrixdir".into()).unwrap();
    for i in 0..5 {
        let now = std::time::SystemTime::UNIX_EPOCH
            .elapsed()
            .unwrap()
            .as_millis();
        matrixdir.write_event(&format!("{i}\n"), now).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
