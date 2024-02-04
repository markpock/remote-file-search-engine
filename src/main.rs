use disk_index::IndexReader;

#[cfg(test)]
mod tests;
mod utils;

mod memory_index;
mod disk_index;
mod search_engine;


fn main() {
    // searchshell::searchshell();
    let memindex = memory_index::crawl("./text").unwrap();
    let path = "test.idx";
    let _ = disk_index::marshal(path, memindex);
    let _ = IndexReader::new(path).unwrap();
}
