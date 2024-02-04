use std::{collections::HashMap, fs::{read_dir, File}, io::{self, Read}, path::{Path, PathBuf}};

use crate::memory_index::memindex::MemIndex;
use crate::utils::*;

pub fn crawl(dir: &str) -> Option<MemIndex> {
    let path = Path::new(dir);
    if path.is_dir() {
        let mut memindex = MemIndex::new();
        let _ = handle_dir(path.to_path_buf(), &mut memindex);
        Some (memindex)
    } else { None }
}

fn handle_dir(dir: PathBuf, memindex: &mut MemIndex) -> Result<(), io::Error> {
    for entry in read_dir(dir)?.filter_map(|e| e.ok()) {
        let file_type = entry.file_type()?;
        if file_type.is_file() {
            let id = memindex.doctable.insert(entry.path());
            for (word, positions) in parse_file(&mut File::open(entry.path())?)? {
                memindex.insert(id, word, positions)
            }
        } else if file_type.is_dir() {
            let _ = handle_dir(entry.path(), memindex);
        }
    }
    Ok (())
}

fn parse_file(f: &mut File) -> Result<HashMap<String, Vec<Offset>>, io::Error> {
    let mut file_as_string = String::new();
    let _ = f.read_to_string(&mut file_as_string)?;

    let mut map: HashMap<String, Vec<Offset>> = HashMap::new();
    let mut intermediate: String = String::new();
    let mut beginning: Offset = 0;

    for (i, ch) in file_as_string.to_lowercase()
        .replace(|c: char| !c.is_alphabetic() && c != ' ' && c != '\n' && c != '\t', "")
        .chars()
        .enumerate()
    {
        if ch != ' ' && ch != '\t' && ch != '\n' { 
            intermediate.push(ch);
            continue;
        }
        if intermediate.len() > 0 {
            match map.get_mut(&intermediate) {
                Some(vec) => vec.push(beginning),
                None => { map.insert(intermediate.clone(), vec![beginning]); }
            }
        }
        beginning = i as Offset;
        intermediate.clear();
    }
    // Do this one more time, since there may not be whitespace at the end
    if intermediate.len() > 0 {
        match map.get_mut(&intermediate) {
            Some(vec) => vec.push(beginning),
            None => { map.insert(intermediate.clone(), vec![beginning]); }
        }
    }
    Ok(map)
}
