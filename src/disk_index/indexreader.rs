use std::{fs::File, io::Read, path::PathBuf};

use crate::{search_engine::{self, SearchIndex}, utils::*};

pub struct IndexReader {
    file: File
}

impl IndexReader {
    pub fn new(path: &str) -> Option<IndexReader> {
        let mut file = File::open(path).ok()?;
        let mut buf = 0usize.to_be_bytes();

        file.read_exact(&mut buf).ok()?;
        let canary = usize::from_be_bytes(buf);
        if canary != 0xDEADBEEFusize { return None }

        file.read_exact(&mut buf).ok()?;
        let checksum = usize::from_be_bytes(buf);
        if checksum != 0xCAFECAFEusize { return None }
        
        Some (IndexReader {file})
    }

    pub fn doc_of_id(&self, id: DocID) -> PathBuf {
        todo!()
    }
}

impl SearchIndex<(PathBuf, usize)> for IndexReader {
    fn search(&self, query: &Vec<&str>) -> Vec<(PathBuf, usize)> {
        search_engine::search(
            query,
            |s| {
                todo!()
            },
            |a, b| a + b,
            usize::cmp
        ).into_iter()
            .map(|(id, len)| (self.doc_of_id(id), len))
            .collect::<Vec<(PathBuf, usize)>>()
    }
}
