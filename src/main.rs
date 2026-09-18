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

    let urls = extract_urls_from_bookmark_html(&content);

    println!("Found {} URLs:", urls.len());
    print_list_of_strings(urls);
}


fn extract_urls_from_bookmark_html(content: &str) -> Vec<String> {
    println!("Parsing {} bytes of HTML...", content.len());
    return content
        .lines()
        .filter_map(|line| {
            let start = line
                .find("HREF=\"")
                .or_else(|| line.find("href=\""))? + 6;
            let rest = &line[start..];
            let (url, _) = rest.split_once('"')?;
            return Some(url.to_string());
        })
        .collect();
}


fn print_list_of_strings(list: Vec<String>) {
    for url in list {
        println!("{}", url);
    }
}