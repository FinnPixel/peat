use std::env;
use std::fs;
use std::process;

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

    parse_netscape_bookmark_html(&content);
}

fn parse_netscape_bookmark_html(content: &str) {
    println!("Parsing {} bytes of HTML...", content.len());
}