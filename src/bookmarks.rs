pub fn extract_urls_from_bookmark_html(content: &str) -> Vec<String> {
    println!("Parsing {} bytes of HTML...\n", content.len());
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
