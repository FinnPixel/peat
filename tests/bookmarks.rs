use peat::bookmarks::{clean_html, extract_urls};
use peat::checker::{CheckResult, LinkStatus};


fn result(url: &str, status: LinkStatus, detail: &str) -> CheckResult {
    CheckResult { url: url.into(), status, detail: detail.into() }
}


fn without_lines_containing(content: &str, urls: &[&str]) -> String {
    content
        .split_inclusive('\n')
        .filter(|line| !urls.iter().any(|url| line.contains(&format!("\"{url}\""))))
        .collect()
}


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


#[test]
fn clean_drops_dead_bookmark_from_fixture() {
    let content = include_str!("fixtures/sample_bookmarks_00.html");
    let results = [
        result("https://example.com", LinkStatus::Alive, "200"),
        result("https://httpbin.org/status/404", LinkStatus::NotFound, "404"),
    ];
    assert_eq!(
        clean_html(content, &results),
        without_lines_containing(content, &["https://httpbin.org/status/404"]),
    );
}


#[test]
fn clean_large_fixture_removes_dead_updates_moved_keeps_rest() {
    let content = include_str!("fixtures/sample_bookmarks_large_00.html");
    let dead = [
        ("https://github.com/this-user-surely-does-not-exist-peat-test", LinkStatus::NotFound),
        ("https://en.wikipedia.org/wiki/This_Page_Does_Not_Exist_Peat_Test", LinkStatus::NotFound),
        ("https://httpbin.org/status/404", LinkStatus::NotFound),
        ("https://httpbin.org/status/410", LinkStatus::NotFound),
        ("https://this-domain-does-not-exist.invalid/", LinkStatus::DnsFailure),
        ("https://old-blog.invalid/posts/2014/hello", LinkStatus::DnsFailure),
        ("http://10.255.255.1/", LinkStatus::Unreachable),
        ("http://localhost:59999/", LinkStatus::Unreachable),
    ];
    let kept = [
        ("https://httpbin.org/status/500", LinkStatus::ServerError),
        ("https://self-signed.badssl.com/", LinkStatus::TlsFailure),
        ("https://1password.com/", LinkStatus::Blocked),
    ];
    let results: Vec<CheckResult> = extract_urls("x.html", content)
        .into_iter()
        .map(|url| {
            let (status, detail) = match url.as_str() {
                "http://rust-lang.org/" => (LinkStatus::Moved, "https://www.rust-lang.org/"),
                _ => dead.iter().chain(&kept)
                    .find(|(u, _)| *u == url)
                    .map_or((LinkStatus::Alive, "200"), |&(_, status)| (status, "")),
            };
            result(&url, status, detail)
        })
        .collect();

    let dead_urls: Vec<&str> = dead.iter().map(|(url, _)| *url).collect();
    let expected = without_lines_containing(content, &dead_urls)
        .replace(r#""http://rust-lang.org/""#, r#""https://www.rust-lang.org/""#);
    assert_eq!(clean_html(content, &results), expected);
}


#[test]
fn clean_rewrites_only_the_href_and_keeps_crlf() {
    let content = r#"<DT><A HREF="https://www.notion.so/" ICON_URI="https://www.notion.so/favicon.ico">Notion</A>
<DT><A HREF="place:sort=8">Recent</A>
"#.replace('\n', "\r\n");
    let results = [result("https://www.notion.so/", LinkStatus::Moved, "https://www.notion.com/")];
    let expected = r#"<DT><A HREF="https://www.notion.com/" ICON_URI="https://www.notion.so/favicon.ico">Notion</A>
<DT><A HREF="place:sort=8">Recent</A>
"#.replace('\n', "\r\n");
    assert_eq!(clean_html(&content, &results), expected);
}


#[test]
fn clean_drops_description_of_dead_bookmark() {
    let content = r#"<DT><A HREF="https://dead.com">Dead</A>
<DD>Dead description
<DT><A HREF="https://alive.com">Alive</A>
<DD>Alive description
"#;
    let results = [
        result("https://alive.com", LinkStatus::Alive, "200"),
        result("https://dead.com", LinkStatus::DnsFailure, "no such host"),
    ];
    let expected = r#"<DT><A HREF="https://alive.com">Alive</A>
<DD>Alive description
"#;
    assert_eq!(clean_html(content, &results), expected);
}
