use indexreader::IndexReader;

mod tests;
mod types;

mod searchindex;

mod crawler;
mod doctable;
mod memindex;
mod parser;
mod searchshell;

mod marshaller;
mod indexreader;

fn main() {
    // searchshell::searchshell();
    let memindex = crawler::crawl("./text").unwrap();
    let path = "test.idx";
    let _ = marshaller::marshal(path, memindex);
    let _ = IndexReader::new(path).unwrap();
}
