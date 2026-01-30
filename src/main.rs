use std::env;
use std::process::Command;

mod provider;
mod tracker;

use provider::SafariProvider;
use tracker::Tracker;

fn main() -> rusqlite::Result<()> {
    let args: Vec<String> = env::args().collect();
    let limit = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(15);
    let filter = args.get(2);

    let mut tracker = Tracker::new();

    for (url, count) in SafariProvider::fetch_history()? {
        tracker.process_record(url, count);
    }

    tracker.display(limit, filter);
    let filename = "report.html";
    if let Err(e) = tracker.export_html("report.html") {
        eprintln!("❌ Не удалось создать HTML-отчет: {}", e);
    } else {
        if let Err(e) = Command::new("open").arg(filename).spawn() {
            eprintln!("❌ Не удалось открыть браузер автоматически: {}", e);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_domain_cleaning() {
        assert_eq!(
            Tracker::clean_domain("https://www.google.com"),
            "google.com"
        );
        assert_eq!(Tracker::clean_domain(".github.com"), "github.com");
    }
}
