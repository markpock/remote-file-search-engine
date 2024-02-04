use crate::searchindex::SearchIndex;

#[test]
pub fn generic() {
    let memindex = crate::crawler::crawl("./text").unwrap();
    println!("{:?}\n\n", memindex);
    for (path, hits) in memindex.search(&vec!["hello", "world"]) {
        println!("{:?}, {}", path, hits);
    }
}
