use super::reader_trait::Reader;
use std::path::Path;

pub struct PDFReader {}

impl Reader for PDFReader {
    fn read_text(file_path: &Path) -> Result<String, ()> {
        let content = pdf_extract::extract_text(&file_path).map_err(|err| {
            eprintln!(
                "ERROR: could not extract text from PDF {file_path}: {err}",
                file_path = file_path.display()
            )
        })?;
        Ok(content)
    }
}
