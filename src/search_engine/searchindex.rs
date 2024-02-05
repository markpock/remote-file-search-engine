use std::{cmp::Ordering, collections::HashMap};

use crate::utils::*;

pub fn search<F, G, H, Rank>(query: &Vec<&str>, ranks: F, combine: G, compare: H) -> Vec<(DocID, Rank)>
    where F : Fn(&str) -> Box<dyn Iterator<Item = (DocID, Rank)>>,
    H : Fn(&Rank, &Rank) -> Ordering,
    G : Fn(&Rank, &Rank) -> Rank {
    let mut documents: HashMap<DocID, Rank> = HashMap::new();
    let mut iter = query.iter();
    if let Some (first) = iter.next() {
        for (doc, val) in &mut ranks(first) {
            documents.insert(doc, val);
        }
    } else { return Vec::new() }
    for word in iter {
        let mut candidates: HashMap<DocID, Rank> = HashMap::new();
        for (doc, val) in &mut ranks(word) {
            if let Some (oldval) = documents.get(&doc) {
                candidates.insert(doc, combine(&val, oldval));
            }
        }
        documents = candidates;
    }
    let mut result = documents.into_iter().collect::<Vec<(DocID, Rank)>>();
    result.sort_by(|(_, rank1), (_, rank2)| compare(rank1, rank2));
    result
}

pub trait SearchIndex<R> {
    fn search(&self, query: &Vec<&str>) -> Vec<R>;
}
