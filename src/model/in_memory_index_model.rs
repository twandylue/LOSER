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
