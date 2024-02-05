use std::{collections::HashMap, path::PathBuf};

use crate::{memory_index::doctable::DocTable, search_engine::{self, SearchIndex}, utils::*};

#[derive(Debug)]
pub struct MemIndex {
    pub(crate) doctable: DocTable,
    pub(crate) index: HashMap<String, HashMap<DocID, Vec<Offset>>>
}

impl MemIndex {
    pub fn new() -> MemIndex {
        MemIndex { doctable: DocTable::new(), index: HashMap::new() }
    }

    pub fn insert(&mut self, id: DocID, word: String, positions: Vec<Offset>) {
        match self.index.get_mut(&word) {
            Some(ht) => {
                ht.insert(id, positions);
            }
            None => {
                let mut ht = HashMap::new();
                ht.insert(id, positions);
                self.index.insert(word, ht);
            }
        }
    }
}

impl SearchIndex<(PathBuf, usize)> for MemIndex {
    fn search(&mut self, query: &Vec<&str>) -> Vec<(PathBuf, usize)> {
        search_engine::search::<_, _, _, usize>(
            query,
            |s| {
                match self.index.get(s) {
                    Some(table) =>
                    Box::new(
                        table.iter()
                        .map(|(&id, vec)| (id, vec.len()))
                        .collect::<Vec<(DocID, usize)>>().into_iter()
                    ),
                    None => Box::new(std::iter::empty())
                }
            },
        |a, b| a + b,
        usize::cmp
        ).into_iter()
            .map(|(id, hits)| (self.doctable.doc(id).unwrap().clone(), hits))
            .collect::<Vec<(PathBuf, usize)>>()
    }
}
