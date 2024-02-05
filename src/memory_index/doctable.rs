use std::{collections::HashMap, path::PathBuf};

use crate::utils::*;


#[derive(Debug)]
pub struct DocTable {
    pub(crate) id_to_doc: Vec<PathBuf>,
    pub(crate) doc_to_id: HashMap<PathBuf, DocID>
}

impl DocTable {
    pub fn new() -> DocTable {
        DocTable {id_to_doc: Vec::new(), doc_to_id: HashMap::new()}
    }

    pub fn insert(&mut self, s: PathBuf) -> DocID {
        self.doc_to_id.insert(s.clone(), self.id_to_doc.len() as DocID);
        self.id_to_doc.push(s);
        self.id_to_doc.len() as DocID - 1
    }

    pub fn id(&self, s: &PathBuf) -> Option<DocID> {
        match self.doc_to_id.get(s) {
            Some(&x) => Some(x),
            None => None
        }
    }

    pub fn doc(&self, i: DocID) -> Option<&PathBuf> {
        self.id_to_doc.get(i as usize)
    }
}
