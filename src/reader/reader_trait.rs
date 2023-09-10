use std::path::Path;

pub trait Reader {
    fn read_text(file_path: &Path) -> Result<String, ()>;
}
