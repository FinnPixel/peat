use std::time::Duration;
use ureq::Agent;


const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36";


pub fn get_live_dead_urls(urls: Vec<String>) -> (Vec<String>, Vec<String>) {
    let agent: Agent = Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(10)))
        .http_status_as_error(false)
        .build()
        .into();
    print!("Checking {} URLs for liveness...\n", urls.len());
    urls.into_iter().partition(|url| is_url_live(&agent, url))
}


fn is_url_live(agent: &Agent, url: &str) -> bool {
    let result = agent
        .get(url)
        .header("User-Agent", USER_AGENT)
        .header("Accept", "text/html")
        .call();

    match result {
        Ok(resp) => {
            let code = resp.status().as_u16();
            !(code == 404 || code == 410 || code >= 500)
        }
        // server accepted the connection, then hung up: it exists, it's blocking bots
        Err(ureq::Error::Io(e)) if e.kind() == std::io::ErrorKind::ConnectionReset => true,
        Err(e) => {
            eprintln!("{url}: {e}");
            false
        }
    }
}
