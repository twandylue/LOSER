use super::super::lexer::Lexer;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

pub trait Model {
    fn add_document(&mut self, file_path: PathBuf, content: &[char]) -> Result<(), ()>;
}

type TermFreq = HashMap<String, usize>;
type FileTF = HashMap<PathBuf, Doc>;
type DocFreq = HashMap<String, usize>;

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Doc {
    tf: TermFreq,
    count: usize,
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct InMemoryIndexModel {
    pub docs: FileTF,
    pub df: DocFreq,
}

impl InMemoryIndexModel {
    pub fn new() -> Self {
        InMemoryIndexModel {
            docs: HashMap::new(),
            df: HashMap::new(),
        }
    }
}

impl Model for InMemoryIndexModel {
    fn add_document(&mut self, file_path: PathBuf, content: &[char]) -> Result<(), ()> {
        let mut tf = TermFreq::new();
        let mut count = 0;

        for token in Lexer::new(content) {
            tf.entry(token).and_modify(|v| *v += 1).or_insert(1);
            count += 1;
        }

        for term in tf.keys() {
            self.df
                .entry(term.to_string())
                .and_modify(|v| *v += 1)
                .or_insert(1);
        }

        let doc = Doc { tf, count };
        self.docs.insert(file_path, doc);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{InMemoryIndexModel, Model};
    use crate::model::in_memory_index_model::Doc;
    use std::{collections::HashMap, path::PathBuf, str::FromStr};

    #[test]
    fn add_document_ok() -> Result<(), ()> {
        // arrange
        let mut model = InMemoryIndexModel::new();
        let path: PathBuf = PathBuf::from_str("test/test.txt")
            .map_err(|err| eprintln!("ERROR: the path is not valid in test: {err}"))?;
        let content = String::from("Andy is Andy.");

        let mut expected = InMemoryIndexModel::new();
        let expected_doc = Doc {
            tf: HashMap::from([
                ("ANDY".to_string(), 2),
                ("IS".to_string(), 1),
                (".".to_string(), 1),
            ]),
            count: 4,
        };
        expected.docs.insert(path.clone(), expected_doc);
        expected.df = HashMap::from([
            ("ANDY".to_string(), 1),
            ("IS".to_string(), 1),
            (".".to_string(), 1),
        ]);

        // act
        model.add_document(path.clone(), &content.chars().collect::<Vec<char>>())?;

        // assert
        assert_eq!(model, expected);

        Ok(())
    }
}
