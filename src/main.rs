use lexer::Lexer;
use reader::plain_text_reader::{PlainTextReader, Reader};
use serde_json;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

mod lexer;
mod reader;

type TermFreq = HashMap<String, usize>;
type FileTF = HashMap<PathBuf, TermFreq>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let paths = std::fs::read_dir("./src/data")?;
    let mut file_tf = FileTF::new();

    println!("Indexing...");
    for path in paths {
        let file_path = path?.path();
        println!("File: {file_path:?}");
        let result = read_from_file(&file_path)?;
        let tf = create_tf(result);

        // let mut sorted_vec = tf.iter().collect::<Vec<(&String, &usize)>>();
        // sorted_vec.sort_by(|a, b| b.1.cmp(a.1));
        //
        // for (k, v) in sorted_vec.iter().take(10) {
        //     println!("  {k} => {v}");
        // }

        file_tf.insert(file_path, tf);
    }

    let output = serde_json::to_string(&file_tf)?;

    std::fs::write("./src/index.json", output)?;

    Ok(())
}

fn read_from_file(file: &Path) -> Result<String, Box<dyn std::error::Error>> {
    match file.extension().unwrap().to_str() {
        Some("txt") => Ok(PlainTextReader::read_text(file)?),
        _ => Err("Error in reader.".into()),
    }
}

fn create_tf(content: String) -> TermFreq {
    let chars = &content.chars().collect::<Vec<char>>();
    let lexer = Lexer::new(chars);

    let mut counter = TermFreq::new();

    lexer.for_each(|x| {
        counter.entry(x).and_modify(|c| *c += 1).or_insert(1);
    });

    counter
}
