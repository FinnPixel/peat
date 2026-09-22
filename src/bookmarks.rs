use std::path::Path;


pub fn extract_urls(file_path: &str, content: &str) -> Vec<String> {
    let is_html = Path::new(file_path)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("html") || ext.eq_ignore_ascii_case("htm"));

    let mut urls = if is_html {
        extract_urls_from_bookmark_html(content)
    } else {
        extract_urls_from_lines(content)
    };
    urls.sort();
    urls.dedup();
    return urls;
}


fn extract_urls_from_bookmark_html(content: &str) -> Vec<String> {
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


fn extract_urls_from_lines(content: &str) -> Vec<String> {
    println!("Parsing {} bytes of text...\n", content.len());
    return content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect();
}
