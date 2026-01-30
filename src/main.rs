use anyhow::Context;
use clap::Parser;
use std::process::Command;

mod config;
mod provider;
mod tracker;

use config::Config;
use provider::SafariProvider;
use tracing::{Level, info};
use tracker::Tracker;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let cfg = Config::parse();

    let mut tracker = Tracker::new();

    let history =
        SafariProvider::fetch_history(cfg.days).context("Не удалось получить данные из Safari")?;

    for (url, count) in history {
        tracker.process_record(url, count);
    }

    tracker.display(cfg.limit, cfg.filter.as_ref());

    let filename = &cfg.output;
    tracker
        .export_html(filename)
        .context("Ошибка при экспорте в HTML")?;
    info!("Отчет успешно сформирован: {}", filename);
    let _ = Command::new("open").arg(filename).spawn();
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
