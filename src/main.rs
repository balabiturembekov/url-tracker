use clap::Parser;
use std::process::Command;

mod config;
mod provider;
mod tracker;

use config::Config;
use provider::SafariProvider;
use tracker::Tracker;

fn main() -> rusqlite::Result<()> {
    let cfg = Config::parse();

    let mut tracker = Tracker::new();

    for (url, count) in SafariProvider::fetch_history(cfg.days)? {
        tracker.process_record(url, count);
    }

    tracker.display(cfg.limit, cfg.filter.as_ref());

    if tracker.export_html(&cfg.output).is_ok() {
        println!("\n✅ Отчет сохранен в {}", cfg.output);
        let _ = Command::new("open").arg(&cfg.output).spawn();
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
