mod config;
mod provider;
mod tracker;

use anyhow::Context;
use clap::Parser;
use config::Config;
use provider::SafariProvider;
use std::process::Command;
use tracing::{Level, info};
use tracker::Tracker;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let cfg = Config::parse();
    let mut tracker = Tracker::new();

    let history =
        SafariProvider::fetch_history(cfg.days).context("Failed to read Safari history")?;

    for record in history {
        tracker.process_record(record);
    }

    tracker.display(cfg.limit, cfg.filter.as_ref());

    tracker.export_html(&cfg.output)?;
    info!("Report generated: {}", cfg.output);
    let _ = Command::new("open").arg(&cfg.output).spawn();

    Ok(())
}
