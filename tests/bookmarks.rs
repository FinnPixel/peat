use peat::bookmarks::extract_urls;


#[test]
fn html_extracts_hrefs() {
    let content = r#"<DT><A HREF="https://b.com">B</A>
<DT><a href="https://a.com">A</a>
<DT><H3>Folder</H3>"#;
    assert_eq!(extract_urls("x.html", content), ["https://a.com", "https://b.com"]);
}


#[test]
fn html_skips_non_http_schemes() {
    let content = r#"<DT><A HREF="javascript:void(0)">JS</A>
<DT><A HREF="place:sort=8&maxResults=10">Recent</A>
<DT><A HREF="chrome://settings">Settings</A>
<DT><A HREF="http://a.com">A</A>"#;
    assert_eq!(extract_urls("x.html", content), ["http://a.com"]);
}


#[test]
fn text_takes_trimmed_non_empty_lines() {
    let content = "  https://a.com  \n\nhttps://b.com\n";
    assert_eq!(extract_urls("x.txt", content), ["https://a.com", "https://b.com"]);
}


#[test]
fn text_skips_non_http_schemes() {
    let content = "javascript:alert(1)\nchrome://flags\nnot a url\nhttps://a.com";
    assert_eq!(extract_urls("x.txt", content), ["https://a.com"]);
}


#[test]
fn output_is_sorted_and_deduped() {
    let content = "https://b.com\nhttps://a.com\nhttps://b.com";
    assert_eq!(extract_urls("x.md", content), ["https://a.com", "https://b.com"]);
}


#[test]
fn md_heading_only_returns_nothing() {
    let content = "# My Bookmarks\n\n## Reading";
    assert!(extract_urls("x.md", content).is_empty());
}


#[test]
fn md_bullet_link() {
    let content = "- [A](https://a.com)\n* https://b.com";
    assert_eq!(extract_urls("x.md", content), ["https://a.com", "https://b.com"]);
}


#[test]
fn md_inline_link_ignores_title() {
    let content = r#"[A](https://a.com "Site A")"#;
    assert_eq!(extract_urls("x.md", content), ["https://a.com"]);
}


#[test]
fn md_two_links_on_one_line() {
    let content = "[A](https://a.com) and [B](https://b.com/path?q=1)";
    assert_eq!(extract_urls("x.md", content), ["https://a.com", "https://b.com/path?q=1"]);
}


#[test]
fn md_bare_url_trims_trailing_punctuation() {
    let content = "See https://a.com, then https://b.com/x. Done!";
    assert_eq!(extract_urls("x.md", content), ["https://a.com", "https://b.com/x"]);
}


#[test]
fn md_autolink() {
    let content = "Visit <https://a.com/page> today";
    assert_eq!(extract_urls("x.md", content), ["https://a.com/page"]);
}


#[test]
fn md_reference_definition() {
    let content = "Read [the docs][d].\n\n[d]: https://a.com/docs \"Docs\"";
    assert_eq!(extract_urls("x.md", content), ["https://a.com/docs"]);
}


#[test]
fn md_skips_non_http_schemes() {
    let content = "[JS](javascript:void(0)) [Mail](mailto:x@y.com) <ftp://a.com> chrome://flags";
    assert!(extract_urls("x.md", content).is_empty());
}
