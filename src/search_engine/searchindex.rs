use std::{collections::HashMap, path::PathBuf};

use crate::utils::*;

pub fn search<F, G, R>(query: &Vec<&str>, getiter: F, mapper: G) -> Vec<R>
    where F : Fn(&str) -> Box<dyn Iterator<Item = (DocID, usize)>>,
    G : Fn(&(DocID, usize)) -> R {
    let mut documents: HashMap<DocID, usize> = HashMap::new();
    let mut iter = query.iter();
    if let Some (first) = iter.next() {
        for (doc, val) in &mut getiter(first) {
            documents.insert(doc, val);
        }
    } else { return Vec::new() }
    for word in iter {
        let mut candidates: HashMap<DocID, usize> = HashMap::new();
        for (doc, val) in &mut getiter(word) {
            if let Some (oldval) = documents.get(&doc) {
                candidates.insert(doc, val + oldval);
            }
        }
        documents = candidates;
    }
    let mut result = documents.into_iter().collect::<Vec<(DocID, usize)>>();
    result.sort_by(|(_, hits1), (_, hits2)| hits2.cmp(hits1));
    result.iter().map(mapper).collect::<Vec<R>>()
}

pub trait SearchIndex {
    fn search(&self, query: &Vec<&str>) -> Vec<(PathBuf, usize)>;
}
