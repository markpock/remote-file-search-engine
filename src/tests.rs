use crate::search_engine::SearchIndex;

#[test]
pub fn generic() {
    let mut memindex = crate::memory_index::crawl("./text").unwrap();
    println!("{:?}\n\n", memindex);
    for (path, hits) in memindex.search(&vec!["hello", "world"]) {
        println!("{:?}, {}", path, hits);
    }
}
