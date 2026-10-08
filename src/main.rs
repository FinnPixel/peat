use std::{env, fs, process};

use peat::{bookmarks, checker, output};
use peat::checker::LinkStatus;

const USAGE: &str = "Usage: peat <path-to-bookmarks.html|.txt|.md> [--clean <output.html>]";

fn main() {
    let args: Vec<String> = env::args().collect();

    let (file_path, clean_path) = match args.as_slice() {
        [_, flag] if flag == "--help" || flag == "-h" => {
            println!("peat {}\nCheck your bookmarks for dead links.\n\n{USAGE}", env!("CARGO_PKG_VERSION"));
            process::exit(0);
        }
        [_, flag] if flag == "--version" || flag == "-V" => {
            println!("peat {}", env!("CARGO_PKG_VERSION"));
            process::exit(0);
        }
        [_, file] => (file, None),
        [_, file, flag, out] if flag == "--clean" => (file, Some(out)),
        _ => {
            eprintln!("{USAGE}");
            process::exit(1);
        }
    };

    if clean_path.is_some() && !bookmarks::is_html(file_path) {
        eprintln!("--clean only works on an exported bookmarks .html file");
        process::exit(1);
    }

    if let Some(out) = clean_path {
        let same_file = fs::canonicalize(out)
            .ok()
            .zip(fs::canonicalize(file_path).ok())
            .is_some_and(|(a, b)| a == b);
        if same_file {
            eprintln!("--clean would overwrite '{file_path}'. Choose a different output file.");
            process::exit(1);
        }
    }

    let content = fs::read_to_string(file_path).unwrap_or_else(|err| {
        eprintln!("Error reading file '{file_path}': {err}");
        process::exit(1);
    });

    let urls = bookmarks::extract_urls(file_path, &content);

    let results = checker::check_all(urls);
    output::print_report(&results);

    if let Some(clean_path) = clean_path {
        let failed = results.iter()
            .filter(|r| matches!(r.status, LinkStatus::DnsFailure | LinkStatus::Unreachable))
            .count();
        if failed * 2 > results.len() {
            eprintln!("Most links failed. Are you offline? Not writing {clean_path}.");
            process::exit(1);
        }
        fs::write(clean_path, bookmarks::clean_html(&content, &results)).unwrap_or_else(|err| {
            eprintln!("Error writing file '{clean_path}': {err}");
            process::exit(1);
        });
        let removed = results.iter().filter(|r| r.status.is_dead()).count();
        let updated = results.iter().filter(|r| r.status == LinkStatus::Moved).count();
        println!("Wrote {clean_path}: removed {removed} dead, updated {updated} moved.");
    }
}
