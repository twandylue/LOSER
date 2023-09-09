use super::reader_trait::Reader;
use std::panic;
use std::path::Path;

pub struct PDFReader {}

impl Reader for PDFReader {
    fn read_text(file_path: &Path) -> Result<String, ()> {
        // NOTE: panic::catch_unwind is used to catch panics from the pdf_extract crate, and I
        // think it's the best way to handle this but in this crate.
        let result = panic::catch_unwind(|| pdf_extract::extract_text(file_path));
        match result {
            Ok(Ok(content)) => Ok(content),
            Ok(Err(err)) => {
                eprintln!(
                    "ERROR: could not extract text from PDF {file_path}: {err}",
                    file_path = file_path.display(),
                    err = err
                );
                Err(())
            }
            Err(_) => {
                eprintln!(
                    "ERROR: could not extract text from PDF {file_path}",
                    file_path = file_path.display()
                );
                Err(())
            }
        }
    }
}
