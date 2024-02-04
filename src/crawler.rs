use std::{fs::{read_dir, File}, io, path::{Path, PathBuf}};

use crate::{parser::parse_file, memindex::MemIndex, doctable::DocTable};

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
