use lexer::Lexer;
use reader::plain_text_reader::{PlainTextReader, Reader};
use std::collections::HashMap;

mod lexer;
mod reader;

type TermFreq = HashMap<String, usize>;
type FileTF = HashMap<String, TermFreq>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let paths = std::fs::read_dir("./src/data")?;
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

        let count = sorted_vec.len();
        for (k, v) in sorted_vec {
            println!("{k} => {v}");
        }

        println!("----------------");
        println!("{f:?}", f = file_path.file_name().unwrap());
        println!("count: {count}");

        let mut file_tf = FileTF::new();
        file_tf.insert(
            // TODO:
            file_path.file_name().unwrap().to_str().unwrap().to_string(),
            counter,
        );
    }

    Ok(())
}
