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
    println!("{}", memindex.doctable.doc(0).unwrap().display());
    let path = "test.idx";
    let _ = memindex.marshal(path).unwrap();
    let mut indexreader = IndexReader::new(path).unwrap();
    for i in 0..6 {
        println!("{:?}", indexreader.doc_of_id(i));
    }
}
