use rusqlite::{Connection, Result};
use std::env;
use std::fs;
use std::path::Path;

pub struct SafariProvider;

impl SafariProvider {
    const DB_PATH: &'static str = "Library/Safari/History.db";
    const TEMP_DB: &'static str = "/tmp/safari_history_copy";

    pub fn fetch_history(days: u32) -> Result<Vec<(String, u32)>> {
        Self::prepare_db_copy()?;

        let conn = Connection::open(Self::TEMP_DB)?;
        let mut stmt = conn.prepare(&format!(
            "
            SELECT i.url, COUNT(v.id)
            FROM history_items i
            JOIN history_visits v ON i.id = v.history_item
            WHERE v.visit_time > (strftime('%s', 'now') - 978307200 - {} * 24 * 3600)
            GROUP BY i.url
            ",
            days
        ))?;

        let rows = stmt.query_map([], |row| {
            let url: String = row.get(0)?;
            let count: u32 = row.get(1)?;
            Ok((url, count))
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }

        Ok(results)
    }

    fn prepare_db_copy() -> Result<()> {
        let home = env::var("HOME").expect("HOME not found!");
        let src = Path::new(&home).join(Self::DB_PATH);

        if let Err(e) = fs::copy(&src, Self::TEMP_DB) {
            eprintln!("❌ Ошибка копирования базы: {}", e);
            return Err(rusqlite::Error::InvalidPath(src));
        }
        Ok(())
    }
}
