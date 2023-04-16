use super::reader_trait::Reader;
use std::{fs, path::Path};

pub struct PlainTextReader {}

impl Reader for PlainTextReader {
    fn read_text(file_path: &Path) -> Result<String, ()> {
        fs::read_to_string(file_path)
            .map_err(|err| eprintln!("ERROR: could not open the file {file_path:?}: {err}"))
    }
}
