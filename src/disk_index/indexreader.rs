use std::{fs::File, io::{Read, Seek, SeekFrom}, mem::size_of, path::PathBuf};

use crate::{search_engine::{self, SearchIndex}, utils::*};

pub struct IndexReader {
    file: File,
    doctable: SeekFrom,
    index: SeekFrom
}

impl IndexReader {
    pub fn new(path: &str) -> Option<IndexReader> {
        let mut file = File::open(path).ok()?;
        let mut buf = 0usize.to_be_bytes();

        file.read_exact(&mut buf).ok()?; // canary

        if usize::from_be_bytes(buf) != 0xDEADBEEFusize { return None }

        file.read_exact(&mut buf).ok()?; // checksum
        if usize::from_be_bytes(buf) != 0xCAFECAFEusize { return None }

        file.read_exact(&mut buf).ok()?; // memindex_bytes
        let doctable_bytes = usize::from_be_bytes(buf);

        file.read_exact(&mut buf).ok()?;

        let doctable = file.stream_position().ok()? as usize;
        
        Some (IndexReader {
            file,
            doctable: SeekFrom::Start(doctable as u64),
            index: SeekFrom::Start((doctable + doctable_bytes) as u64)
        })
    }

    pub fn doc_of_id(&mut self, id: DocID) -> Option<PathBuf> {
        self.file.seek(self.doctable).ok()?;

        let mut buf = 0usize.to_be_bytes();

        self.file.read_exact(&mut buf).ok()?; // vector offset
        self.file.read_exact(&mut buf).ok()?; // hashtable offset
        self.file.read_exact(&mut buf).ok()?;
        let vector_len = usize::from_be_bytes(buf);

        if id as usize >= vector_len { return None }

        self.file.seek(SeekFrom::Current((id as usize * size_of::<usize>()) as i64)).ok()?;

        self.file.read_exact(&mut buf).ok()?; // absolute offset of string
        self.file.seek(SeekFrom::Start(usize::from_be_bytes(buf) as u64)).ok()?;

        self.file.read_exact(&mut buf).ok()?; // strlen
        let mut strbuf = vec![0; usize::from_be_bytes(buf)];
        self.file.read_exact(strbuf.as_mut_slice()).ok(); // string itself

        Some(PathBuf::from(String::from_utf8(strbuf).unwrap()))
    }
}

impl SearchIndex<(PathBuf, usize)> for IndexReader {
    fn search(&mut self, query: &Vec<&str>) -> Vec<(PathBuf, usize)> {
        search_engine::search(
            query,
            |s| {
                todo!()
            },
            |a, b| a + b,
            usize::cmp
        ).into_iter()
            .map(|(id, len)| (self.doc_of_id(id).unwrap(), len))
            .collect::<Vec<(PathBuf, usize)>>()
    }
}
