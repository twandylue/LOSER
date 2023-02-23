use lexer::Lexer;
use reader::plain_text_reader::{PlainTextReader, Reader};
use std::path::Path;

mod lexer;
mod reader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = Path::new("./src/data/test.txt");

    let result = PlainTextReader::read_text(file_path)?;
    let chars = &result.chars().collect::<Vec<char>>();
    let lexer = Lexer::new(chars);

    println!("{:?}", lexer.content);

    Ok(())
}
