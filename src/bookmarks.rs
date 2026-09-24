use std::path::Path;

const URL_TERMINATORS: [char; 5] = [')', '>', ']', '"', '\''];
const TRAILING_PUNCTUATION: [char; 6] = ['.', ',', ';', ':', '!', '?'];


pub fn extract_urls(file_path: &str, content: &str) -> Vec<String> {
    let extension = Path::new(file_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(str::to_ascii_lowercase);

    let mut urls = match extension.as_deref() {
        Some("html" | "htm") => extract_urls_from_bookmark_html(content),
        Some("md" | "markdown") => extract_urls_from_markdown(content),
        _ => extract_urls_from_lines(content),
    };
    urls.sort();
    urls.dedup();
    urls
}


fn extract_urls_from_bookmark_html(content: &str) -> Vec<String> {
    println!("Parsing {} bytes of HTML...\n", content.len());
    content
        .lines()
        .filter_map(|line| {
            let start = line
                .find("HREF=\"")
                .or_else(|| line.find("href=\""))? + 6;
            let rest = &line[start..];
            let (url, _) = rest.split_once('"')?;
            Some(url.to_string())
        })
        .filter(|url| is_http_url(url))
        .collect()
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


fn is_http_url(url: &str) -> bool {
    ["http://", "https://"].iter().any(|scheme| {
        url.len() > scheme.len()
            && url.get(..scheme.len()).is_some_and(|prefix| prefix.eq_ignore_ascii_case(scheme))
    })
}
