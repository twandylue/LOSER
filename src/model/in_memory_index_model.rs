use super::super::lexer::Lexer;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, time::SystemTime};

pub trait Model {
    fn add_document(
        &mut self,
        file_path: PathBuf,
        content: &[char],
        last_modified: SystemTime,
    ) -> Result<(), ()>;

    fn remove_document(&mut self, file_path: &PathBuf);

    fn search(&self, query: &[char]) -> Result<Vec<(PathBuf, f32)>, ()>;

    fn requires_reindexing(&self, file_path: &PathBuf, last_modified: SystemTime) -> bool;
}

type TermFreq = HashMap<String, usize>;
type FileTF = HashMap<PathBuf, Doc>;
// NOTE: DocFreq: Appearing times of different tokens in different docs
type DocFreq = HashMap<String, usize>;

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Doc {
    tf: TermFreq,
    total_tokens: usize,
    last_modified: SystemTime,
}

#[derive(Default, Debug, PartialEq, Eq, Deserialize, Serialize)]
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
    let m = doc.total_tokens as f32;
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

    fn add_document(
        &mut self,
        file_path: PathBuf,
        content: &[char],
        last_modified: SystemTime,
    ) -> Result<(), ()> {
        self.remove_document(&file_path);

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

        let doc = Doc {
            tf,
            total_tokens: count,
            last_modified,
        };
        self.docs.insert(file_path, doc);

        Ok(())
    }

    fn requires_reindexing(&self, file_path: &PathBuf, last_modified: SystemTime) -> bool {
        if let Some(doc) = self.docs.get(file_path) {
            return doc.last_modified < last_modified;
        }

        return true;
    }

    fn remove_document(&mut self, file_path: &PathBuf) {
        if let Some(doc) = self.docs.remove(file_path) {
            for t in doc.tf.keys() {
                if let Some(f) = self.df.get_mut(t) {
                    *f -= 1;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::in_memory_index_model::Doc;
    use super::{InMemoryIndexModel, Model};
    use std::{
        collections::HashMap,
        ops::Add,
        path::PathBuf,
        str::FromStr,
        time::{Duration, SystemTime},
    };

    #[test]
    fn add_document_ok() -> Result<(), ()> {
        // arrange
        let mut model = InMemoryIndexModel::new();
        let path: PathBuf = PathBuf::from_str("test/test.txt")
            .map_err(|err| eprintln!("ERROR: the path is not valid in test: {err}"))?;
        let content = String::from("Andy is Andy.");
        let time = SystemTime::now();

        let mut expected = InMemoryIndexModel::new();
        let expected_doc = Doc {
            tf: HashMap::from([
                ("ANDY".to_string(), 2),
                ("IS".to_string(), 1),
                (".".to_string(), 1),
            ]),
            total_tokens: 4,
            last_modified: time,
        };
        expected.docs.insert(path.clone(), expected_doc);
        expected.df = HashMap::from([
            ("ANDY".to_string(), 1),
            ("IS".to_string(), 1),
            (".".to_string(), 1),
        ]);

        // act
        model.add_document(path.clone(), &content.chars().collect::<Vec<char>>(), time)?;

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
        model.add_document(
            path1.clone(),
            &content1.chars().collect::<Vec<char>>(),
            SystemTime::now(),
        )?;
        let path2: PathBuf = PathBuf::from_str("test/test2.txt")
            .map_err(|err| eprintln!("ERROR: the path is not valid in test: {err}"))?;
        let content2 = String::from("Amy is Amy.");
        model.add_document(
            path2.clone(),
            &content2.chars().collect::<Vec<char>>(),
            SystemTime::now(),
        )?;

        let value1 = 0.5_f32 * (2_f32 / 1_f32).log10(); // tf * idf
        let expected = vec![(path1.clone(), value1), (path2.clone(), 0_f32)];

        // act
        let query = "andy".to_string().chars().collect::<Vec<_>>();
        let actual = model.search(&query)?;

        // assert
        assert_eq!(expected, actual);

        Ok(())
    }

    #[test]
    fn requires_reindexing_ok() -> Result<(), ()> {
        // arrange
        let mut model = InMemoryIndexModel::new();
        let file_path: PathBuf = PathBuf::from_str("test/test.txt")
            .map_err(|err| eprintln!("ERROR: the path is not valid in test: {err}"))?;
        let content = String::from("Andy is Andy.");
        let time = SystemTime::now();

        model.add_document(
            file_path.clone(),
            &content.chars().collect::<Vec<char>>(),
            time,
        )?;

        // act
        let actual = model.requires_reindexing(&file_path, time.add(Duration::from_secs(10)));

        // assert
        assert!(actual);

        Ok(())
    }

    #[test]
    fn remove_document_ok() -> Result<(), ()> {
        // arrange
        let mut model = InMemoryIndexModel::new();
        let file_path1: PathBuf = PathBuf::from_str("test/test1.txt")
            .map_err(|err| eprintln!("ERROR: the path is not valid in test: {err}"))?;
        let content1 = String::from("Andy is Andy.");
        let file_path2: PathBuf = PathBuf::from_str("test/test2.txt")
            .map_err(|err| eprintln!("ERROR: the path is not valid in test: {err}"))?;
        let content2 = String::from("Amy is Amy.");
        let time = SystemTime::now();

        model.add_document(
            file_path1.clone(),
            &content1.chars().collect::<Vec<char>>(),
            time,
        )?;

        model.add_document(
            file_path2.clone(),
            &content2.chars().collect::<Vec<char>>(),
            time,
        )?;

        // act && assert
        assert_eq!(model.docs.keys().count(), 2);
        assert_eq!(
            model.df.get("ANDY").and_then(|count| Some(*count as i32)),
            Some(1)
        );
        assert_eq!(
            model.df.get("IS").and_then(|count| Some(*count as i32)),
            Some(2)
        );

        model.remove_document(&file_path1);

        assert_eq!(model.docs.keys().count(), 1);
        assert_eq!(
            model.df.get("ANDY").and_then(|count| Some(*count as i32)),
            Some(0)
        );
        assert_eq!(
            model.df.get("IS").and_then(|count| Some(*count as i32)),
            Some(1)
        );

        Ok(())
    }
}
