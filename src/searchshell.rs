use std::io::stdin;

use crate::{searchindex::SearchIndex};

pub fn searchshell(index: impl SearchIndex) {
    println!("Welcome to the file search shell! Indexing folder...");
    println!("Indexing complete. Type 'quit()' to quit. Input your query on one line.\n");
    loop {
        let mut buffer = String::new();
        match stdin().read_line(&mut buffer) {
            Ok(_) => {
                buffer = buffer.trim_end().to_string().to_lowercase();
                if buffer == "quit()" {
                    println!("  Shutting down.");
                    return
                }
                let query = buffer.split_ascii_whitespace().collect::<Vec<&str>>();
                let result = index.search(&query);
                if result.len() > 0 {
                    for (path, hits) in result {
                        println!("  {} => {}", path.display(), hits);
                    }
                } else {
                    println!("  No results found.")
                }
            }
            Err(_) => {
                println!("  Could not parse line.. aborting.");
                return
            }
        }
        buffer.clear();
        println!()
    }
}