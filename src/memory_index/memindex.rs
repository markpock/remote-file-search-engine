use std::{collections::HashMap, fs::File, io::{self, Write}, path::PathBuf};

use crate::{memory_index::doctable::DocTable, utils::write_hashtable, search_engine::{self, SearchIndex}, utils::*};

#[derive(Debug)]
pub struct MemIndex {
    pub(crate) doctable: DocTable,
    index: HashMap<String, HashMap<DocID, Vec<Offset>>>
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

    pub fn write_to_file(&self, file: &mut File) -> Result<usize, io::Error> {
        Ok (
            self.doctable.write_to_file(file)? +
            write_hashtable(file,
                &self.index,
                |f, k| {
                    Ok(f.write(k.len().to_be_bytes().as_slice())? + f.write(k.as_bytes())?)
                },
                |f, v| write_hashtable(
                    f, v,
                    |f_, k| f_.write(k.to_be_bytes().as_slice()),
                    |f_, v_| {
                        let mut accum = 0usize;
                        for &elt in v_ {
                            accum += f_.write(elt.to_be_bytes().as_slice())?;
                        }
                        Ok(accum)
                    }
                )
            )?
        )
    }
}

impl SearchIndex for MemIndex {
    fn search(&self, query: &Vec<&str>) -> Vec<(PathBuf, usize)> {
        search_engine::search(query, |s| {
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
        |(id, hits)| (self.doctable.doc(*id).unwrap().clone(), *hits))
    }
}
