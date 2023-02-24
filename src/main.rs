use lexer::Lexer;
use reader::plain_text_reader::{PlainTextReader, Reader};
use serde_json;
use std::{collections::HashMap, path::PathBuf};

mod lexer;
mod reader;

type TermFreq = HashMap<String, usize>;
type FileTF = HashMap<PathBuf, TermFreq>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let paths = std::fs::read_dir("./src/data")?;
    let mut file_tf = FileTF::new();

    for path in paths {
        let file_path = path?.path();

        let result = PlainTextReader::read_text(&file_path)?;
        let chars = &result.chars().collect::<Vec<char>>();
        let lexer = Lexer::new(chars);

        let mut counter = TermFreq::new();

        lexer.for_each(|x| {
            counter.entry(x).and_modify(|c| *c += 1).or_insert(1);
        });

        let mut sorted_vec = counter.iter().collect::<Vec<(&String, &usize)>>();
        sorted_vec.sort_by(|a, b| b.1.cmp(a.1));

        println!("{file_path:?}");
        for (k, v) in sorted_vec.iter().take(10) {
            println!("  {k} => {v}");
        }

        file_tf.insert(file_path, counter);
        println!();
    }

    let output = serde_json::to_string(&file_tf)?;
    println!("{output}");

    std::fs::write("./src/index.json", output)?;

    Ok(())
}
