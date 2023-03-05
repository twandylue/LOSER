use std::{fs, path::Path};

pub trait Reader {
    fn read_text(file_path: &Path) -> Result<String, ()>;
}

pub struct PlainTextReader {}

impl Reader for PlainTextReader {
    fn read_text(file_path: &Path) -> Result<String, ()> {
        fs::read_to_string(file_path)
            .map_err(|err| eprintln!("ERROR: could not open the file {file_path:?}: {err}"))
    }
}
