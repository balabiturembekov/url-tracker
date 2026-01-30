use anyhow::{Context, Result};
use rusqlite::Connection;
use std::env;
use std::fs;
use std::path::Path;
use tracing::{info, warn};

pub struct VisitRecord {
    pub url: String,
    pub count: u32,
    pub hour: usize,
}

pub struct SafariProvider;

impl SafariProvider {
    const DB_PATH: &'static str = "Library/Safari/History.db";
    const TEMP_DB: &'static str = "/tmp/safari_history_copy";

    pub fn fetch_history(days: u32) -> Result<Vec<VisitRecord>> {
        Self::prepare_db_copy().context("Failed to copy Safari DB")?;

        let conn = Connection::open(Self::TEMP_DB)?;

        let mut stmt = conn.prepare(&format!("
            SELECT 
                i.url, 
                COUNT(v.id),
                CAST(STRFTIME('%H', v.visit_time + 978307200, 'unixepoch', 'localtime') AS INTEGER) as hour
            FROM history_items i
            JOIN history_visits v ON i.id = v.history_item
            WHERE v.visit_time > (strftime('%s', 'now') - 978307200 - {} * 24 * 3600)
            GROUP BY i.url, hour
        ", days))?;

        let rows = stmt.query_map([], |row| {
            Ok(VisitRecord {
                url: row.get(0)?,
                count: row.get(1)?,
                hour: row.get::<_, u32>(2)? as usize,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }

        info!("Loaded {} records from Safari", results.len());
        Ok(results)
    }

    fn prepare_db_copy() -> Result<()> {
        let home = env::var("HOME").context("HOME not set")?;
        let src = Path::new(&home).join(Self::DB_PATH);
        if let Err(e) = fs::copy(&src, Self::TEMP_DB) {
            warn!("Could not copy Safari DB. Check Full Disk Access.");
            return Err(anyhow::anyhow!("Copy failed: {}", e));
        }
        Ok(())
    }
}
