use std::{fs::File, io::Read, path::Path};

type Error = Box<dyn std::error::Error>;

pub trait Reader {
    fn read_text(file_path: &Path) -> Result<String, Error>;
}

pub struct PlainTextReader {}

impl Reader for PlainTextReader {
    fn read_text(file_path: &Path) -> Result<String, Error> {
        let mut buf = String::new();
        let mut file = File::open(file_path)?;
        file.read_to_string(&mut buf)?;

        Ok(buf)
    }
}
