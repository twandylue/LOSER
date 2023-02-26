use super::super::lexer::Lexer;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

pub trait Model {
    fn add_document(&mut self, file_path: PathBuf, content: &[char]) -> Result<(), ()>;
    fn search(&self, query: &[char]) -> Result<Vec<(PathBuf, f32)>, ()>;
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

fn compute_tf(token: &str, doc: &Doc) -> f32 {
    let m = doc.count as f32;
    let n = doc.tf.get(token).cloned().unwrap_or(0) as f32;
    n / m
}

fn compute_idf(token: &str, model: &InMemoryIndexModel) -> f32 {
    let n = model.docs.len() as f32;
    let m = model.df.get(token).cloned().unwrap_or(1) as f32;
    (n / m).log10()
}

impl Model for InMemoryIndexModel {
    fn search(&self, query: &[char]) -> Result<Vec<(PathBuf, f32)>, ()> {
        let mut result: Vec<(PathBuf, f32)> = Vec::new();
        let tokens = Lexer::new(query).collect::<Vec<String>>();
        for (path, doc) in &self.docs {
            let mut rank = 0_f32;
            for token in &tokens {
                rank += compute_tf(token.as_str(), doc) * compute_idf(token.as_str(), &self);
            }
            result.push((path.to_path_buf(), rank));
        }
        result.sort_by(|(_, rank1), (_, rank2)| rank2.partial_cmp(rank1).unwrap());
        Ok(result)
    }

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

    #[test]
    fn search_ok() -> Result<(), ()> {
        // arrange
        let mut model = InMemoryIndexModel::new();
        let path1: PathBuf = PathBuf::from_str("test/test.txt")
            .map_err(|err| eprintln!("ERROR: the path is not valid in test: {err}"))?;
        let content1 = String::from("Andy is Andy.");
        model.add_document(path1.clone(), &content1.chars().collect::<Vec<char>>())?;
        let path2: PathBuf = PathBuf::from_str("test/test2.txt")
            .map_err(|err| eprintln!("ERROR: the path is not valid in test: {err}"))?;
        let content2 = String::from("Amy is Amy.");
        model.add_document(path2.clone(), &content2.chars().collect::<Vec<char>>())?;

        let value1 = 0.5_f32 * (2_f32 / 1_f32).log10(); // tf * idf
        let expected = vec![(path1.clone(), value1), (path2.clone(), 0_f32)];

        // act
        let query = "andy".to_string().chars().collect::<Vec<_>>();
        let actual = model.search(&query)?;

        // assert
        assert_eq!(expected, actual);

        Ok(())
    }
}
