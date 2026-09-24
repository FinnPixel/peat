use crate::checker::{CheckResult, LinkStatus};

const ORDER: [LinkStatus; 8] = [
    LinkStatus::NotFound,
    LinkStatus::DnsFailure,
    LinkStatus::Unreachable,
    LinkStatus::TlsFailure,
    LinkStatus::ServerError,
    LinkStatus::Blocked,
    LinkStatus::Moved,
    LinkStatus::Alive,
];


pub fn print_report(results: &[CheckResult]) {
    println!("------------ REPORT ------------");
    println!("Total: {} URLs.\n", results.len());

    for status in ORDER {
        let group: Vec<&CheckResult> =
            results.iter().filter(|r| r.status == status).collect();

        if group.is_empty() {
            continue;
        }

        println!("{} ({}):", status.label(), group.len());
        for result in group {
            if status == LinkStatus::Alive {
                println!("  {}", result.url);
            } else if status == LinkStatus::Moved {
                println!("  {} → {}", result.url, result.detail);
            } else {
                println!("  {} ({})", result.url, result.detail);
            }
        }
        println!();
    }
}