use std::collections::HashMap;
use std::ops::Range;
use std::path::Path;

use crate::checker::{CheckResult, LinkStatus};

const URL_TERMINATORS: [char; 5] = [')', '>', ']', '"', '\''];
const TRAILING_PUNCTUATION: [char; 6] = ['.', ',', ';', ':', '!', '?'];


pub fn extract_urls(file_path: &str, content: &str) -> Vec<String> {
    let mut urls = match extension(file_path).as_deref() {
        Some("html" | "htm") => extract_urls_from_bookmark_html(content),
        Some("md" | "markdown") => extract_urls_from_markdown(content),
        _ => extract_urls_from_lines(content),
    };
    urls.sort();
    urls.dedup();
    urls
}


pub fn is_html(file_path: &str) -> bool {
    matches!(extension(file_path).as_deref(), Some("html" | "htm"))
}


pub fn clean_html(content: &str, results: &[CheckResult]) -> String {
    let by_url: HashMap<&str, &CheckResult> =
        results.iter().map(|r| (r.url.as_str(), r)).collect();
    let mut cleaned = String::with_capacity(content.len());
    let mut dropped_previous = false;

    for line in content.split_inclusive('\n') {
        if dropped_previous && starts_with_ignore_case(line.trim_start(), "<DD>") {
            continue;
        }
        dropped_previous = false;

        let Some(href) = href_range(line) else {
            cleaned.push_str(line);
            continue;
        };
        match by_url.get(unescape_html(&line[href.clone()]).as_str()) {
            Some(result) if result.status.is_dead() => dropped_previous = true,
            Some(result) if result.status == LinkStatus::Moved => {
                cleaned.push_str(&line[..href.start]);
                cleaned.push_str(&escape_html(&result.detail));
                cleaned.push_str(&line[href.end..]);
            }
            _ => cleaned.push_str(line),
        }
    }
    cleaned
}


fn extension(file_path: &str) -> Option<String> {
    Path::new(file_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(str::to_ascii_lowercase)
}


fn extract_urls_from_bookmark_html(content: &str) -> Vec<String> {
    println!("Parsing {} bytes of HTML...\n", content.len());
    content
        .lines()
        .filter_map(|line| Some(unescape_html(&line[href_range(line)?])))
        .filter(|url| is_http_url(url))
        .collect()
}


fn unescape_html(s: &str) -> String {
    s.replace("&quot;", "\"")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
}


fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}


fn href_range(line: &str) -> Option<Range<usize>> {
    let start = line
        .find("HREF=\"")
        .or_else(|| line.find("href=\""))? + 6;
    let len = line[start..].find('"')?;
    Some(start..start + len)
}


fn extract_urls_from_markdown(content: &str) -> Vec<String> {
    println!("Parsing {} bytes of Markdown...\n", content.len());
    content.lines().flat_map(scan_http_urls).collect()
}


fn extract_urls_from_lines(content: &str) -> Vec<String> {
    println!("Parsing {} bytes of text...\n", content.len());
    content
        .lines()
        .map(str::trim)
        .filter(|line| is_http_url(line))
        .map(str::to_string)
        .collect()
}


fn scan_http_urls(line: &str) -> Vec<String> {
    let lower = line.to_ascii_lowercase();
    let mut urls = Vec::new();
    let mut pos = 0;

    while let Some(found) = lower[pos..].find("http") {
        let start = pos + found;
        let end = line[start..]
            .find(|c: char| c.is_whitespace() || URL_TERMINATORS.contains(&c))
            .map_or(line.len(), |i| start + i);
        let url = line[start..end].trim_end_matches(TRAILING_PUNCTUATION);

        if is_http_url(url) {
            urls.push(url.to_string());
            pos = end;
        } else {
            pos = start + "http".len();
        }
    }
    urls
}


fn starts_with_ignore_case(text: &str, prefix: &str) -> bool {
    text.get(..prefix.len()).is_some_and(|start| start.eq_ignore_ascii_case(prefix))
}


fn is_http_url(url: &str) -> bool {
    ["http://", "https://"].iter().any(|scheme| {
        url.len() > scheme.len() && starts_with_ignore_case(url, scheme)
    })
}
