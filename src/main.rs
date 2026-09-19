use std::{env, fs, process};

mod bookmarks;
mod checker;
mod output;


fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: peat <path-to-bookmarks.html>");
        process::exit(1);
    }

    let file_path = &args[1];
    
    let content = fs::read_to_string(file_path).unwrap_or_else(|err| {
        eprintln!("Error reading file '{file_path}': {err}");
        process::exit(1);
    });

    let mut urls = bookmarks::extract_urls_from_bookmark_html(&content);
    urls.sort();
    urls.dedup();

    let (live_urls, dead_urls) = checker::get_live_dead_urls(urls);

    println!("Found {} URLs:\n", live_urls.len() + dead_urls.len());
    println!("Live ({}):", live_urls.len());
    output::print_list_of_strings(live_urls);
    println!("Dead ({}):", dead_urls.len());
    output::print_list_of_strings(dead_urls);
}
