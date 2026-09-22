use std::sync::{Mutex};
use std::thread;
use std::time::Duration;
use ureq::Agent;

const USER_AGENT: &str = "peat/0.1 (+https://github.com/you/peat)";

const WORKERS: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkStatus {
    Alive,
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