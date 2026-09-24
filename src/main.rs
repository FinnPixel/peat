use std::{env, fs, process};

use peat::{bookmarks, checker, output};


fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: peat <path-to-bookmarks.html|.txt|.md>");
        process::exit(1);
    }

    let file_path = &args[1];
    
    let content = fs::read_to_string(file_path).unwrap_or_else(|err| {
        eprintln!("Error reading file '{file_path}': {err}");
        process::exit(1);
    });

    let urls = bookmarks::extract_urls(file_path, &content);

    let results = checker::check_all(urls);
    output::print_report(&results);
}
