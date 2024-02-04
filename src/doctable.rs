use std::{collections::HashMap, fs::File, io::{self, Seek, SeekFrom, Write}, mem::size_of, path::PathBuf};

use crate::{marshaller::write_hashtable, types::*};


#[derive(Debug)]
pub struct DocTable {
    id_to_doc: Vec<PathBuf>,
    doc_to_id: HashMap<PathBuf, DocID>
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

    pub fn write_to_file(&self, file: &mut File) -> Result<usize, io::Error> {
        // write offsets
        let original_position = file.stream_position()? as usize;
        let mut header_position = original_position;
        let mut vector_pos = original_position + 2 * size_of::<usize>();
        header_position += file.write(vector_pos.to_be_bytes().as_slice())?;
        file.seek(SeekFrom::Start(vector_pos as u64))?;
        // write id_to_doc
        let len = self.id_to_doc.len();
        vector_pos += file.write(len.to_be_bytes().as_slice())?;
        let mut element_pos = vector_pos + len * size_of::<usize>();
        for i in 0..len {
            vector_pos += file.write(element_pos.to_be_bytes().as_slice())?;
            file.seek(SeekFrom::Start(element_pos as u64))?;
            let str = self.id_to_doc[i].to_str().unwrap_or_else(|| "");
            element_pos += file.write(str.len().to_be_bytes().as_slice())?;
            element_pos += file.write(str.as_bytes())?;
        }
        file.seek(SeekFrom::Start(header_position as u64))?;
        header_position += file.write(element_pos.to_be_bytes().as_slice())?;
        let ht_bytes = write_hashtable(
            file, 
            &self.doc_to_id, 
            |f, k| {
                let str = k.to_str().unwrap();
                Ok(f.write(str.len().to_be_bytes().as_slice())? + f.write(str.as_bytes())?)
            },
            |f, v| f.write(v.to_be_bytes().as_slice())
        )?;
        Ok(ht_bytes + element_pos - original_position)
    }
}
