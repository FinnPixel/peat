use std::sync::{Mutex};
use std::thread;
use std::time::Duration;
use ureq::{Agent, ResponseExt};

const USER_AGENT: &str = concat!("peat/", env!("CARGO_PKG_VERSION"), " (+https://github.com/FinnPixel/peat)");

const WORKERS: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkStatus {
    Alive,
    Moved,
    NotFound,
    Blocked,
    ServerError,
    DnsFailure,
    TlsFailure,
    Unreachable,
}

impl LinkStatus {
    pub fn label(&self) -> &'static str {
        match self {
            LinkStatus::Alive => "alive",
            LinkStatus::Moved => "moved",
            LinkStatus::NotFound => "not found",
            LinkStatus::Blocked => "blocked",
            LinkStatus::ServerError => "server error",
            LinkStatus::DnsFailure => "dns failure",
            LinkStatus::TlsFailure => "tls failure",
            LinkStatus::Unreachable => "unreachable",
        }
    }
}

#[derive(Debug, Clone)]
pub struct CheckResult {
    pub url: String,
    pub status: LinkStatus,
    pub detail: String,
}


pub fn check_all(mut urls: Vec<String>) -> Vec<CheckResult> {
    println!("Checking {} urls ...\n", urls.len());
    urls.sort_unstable();
    urls.dedup();
 
    let agent: Agent = Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(10)))
        .http_status_as_error(false)
        .build()
        .into();
 
    let workers = WORKERS.min(urls.len());
    let queue = Mutex::new(urls.into_iter());
    let next_url = || queue.lock().unwrap().next();
 
    let mut results: Vec<CheckResult> = thread::scope(|scope| {
        let handles: Vec<_> = (0..workers)
            .map(|_| {
                scope.spawn(|| {
                    let mut checked = Vec::new();
                    while let Some(url) = next_url() {
                        let (status, detail) = check_url(&agent, &url);
                        checked.push(CheckResult { url, status, detail });
                    }
                    checked
                })
            })
            .collect();
 
        handles
            .into_iter()
            .flat_map(|handle| handle.join().unwrap())
            .collect()
    });
 
    results.sort_unstable_by(|a, b| a.url.cmp(&b.url));
    results
}


fn check_url(agent: &Agent, url: &str) -> (LinkStatus, String) {
    let mut attempt = 0;

    loop {
        let (status, detail) = attempt_once(agent, url);
        let transient = matches!(status, LinkStatus::ServerError | LinkStatus::Unreachable)
            || (status == LinkStatus::Blocked && detail == "connection reset");

        if transient && attempt < 1 {
            attempt += 1;
            thread::sleep(Duration::from_millis(750));
            continue;
        }

        return (status, detail);
    }
}


fn attempt_once(agent: &Agent, url: &str) -> (LinkStatus, String) {
    match agent
        .get(url)
        .header("User-Agent", USER_AGENT)
        .header("Accept", "text/html")
        .call()
    {
        Ok(resp) => {
            let code = resp.status().as_u16();
            let status = match code {
                404 | 410 => LinkStatus::NotFound,
                401 | 403 | 429 => LinkStatus::Blocked,
                500..=599 => LinkStatus::ServerError,
                _ => LinkStatus::Alive,
            };
            let final_url = resp.get_uri().to_string();
            if status == LinkStatus::Alive && is_real_move(url, &final_url) {
                return (LinkStatus::Moved, final_url);
            }
            (status, code.to_string())
        }

        Err(ureq::Error::Io(e)) if e.kind() == std::io::ErrorKind::ConnectionReset => {
            (LinkStatus::Blocked, "connection reset".into())
        }

        Err(e) => {
            let detail = e.to_string();
            (classify_error(&detail), detail)
        }
    }
}


fn is_real_move(original: &str, final_url: &str) -> bool {
    let (orig_host, orig_path) = host_and_path(original);
    let (final_host, final_path) = host_and_path(final_url);
    if site(&orig_host) != site(&final_host) {
        return true;
    }
    let orig_path = orig_path.trim_end_matches('/');
    !orig_path.is_empty() && orig_path != final_path.trim_end_matches('/')
}


fn host_and_path(url: &str) -> (String, &str) {
    let url = url.split(['#', '?']).next().unwrap_or(url);
    let rest = url.split_once("://").map_or(url, |(_, rest)| rest);
    let (authority, path) = rest.find('/').map_or((rest, ""), |i| rest.split_at(i));
    let host = authority.rsplit('@').next().unwrap_or(authority);
    let host = host.split(':').next().unwrap_or(host);
    (host.to_ascii_lowercase(), path)
}


fn site(host: &str) -> String {
    let mut labels: Vec<&str> = host.trim_end_matches('.').rsplit('.').take(2).collect();
    labels.reverse();
    labels.join(".")
}


fn classify_error(detail: &str) -> LinkStatus {
    let lowered = detail.to_ascii_lowercase();

    const DNS: [&str; 4] = [
        "no such host",            // Windows
        "name or service not known", // Linux
        "nodename nor servname",   // macOS
        "failed to lookup",
    ];
    if DNS.iter().any(|needle| lowered.contains(needle)) {
        return LinkStatus::DnsFailure;
    }

    const TLS: [&str; 4] = ["certificate", "unknownissuer", "tls", "handshake"];
    if TLS.iter().any(|needle| lowered.contains(needle)) {
        return LinkStatus::TlsFailure;
    }

    LinkStatus::Unreachable
}