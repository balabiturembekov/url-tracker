use rusqlite::{Connection, Result};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

struct Tracker {
    counts: HashMap<String, u32>,
}

impl Tracker {
    fn new() -> Self {
        Tracker {
            counts: HashMap::new(),
        }
    }

    fn process_record(&mut self, url: String, count: u32) {
        let domain = url
            .split('/')
            .nth(2)
            .unwrap_or(&url)
            .trim_start_matches('.')
            .replace("www.", "")
            .to_lowercase();

        *self.counts.entry(domain).or_insert(0) += count;
    }

    fn display(&self, limit: usize, filter: Option<&String>) {
        let mut sorted: Vec<_> = self
            .counts
            .iter()
            .filter(|(domain, _)| filter.map_or(true, |f| domain.contains(f)))
            .collect();

        sorted.sort_by(|a, b| b.1.cmp(a.1));

        let max_val = sorted.first().map(|x| *x.1).unwrap_or(1);

        println!("\n📊 TOP {} MOST VISITED (last 7 days):", limit);
        println!("{:<30} | {:<10} | {:<20}", "Domain", "Visits", "Graph");
        println!("{}", "-".repeat(65));

        for (domain, count) in sorted.into_iter().take(limit) {
            let bar_len = (count * 20 / max_val) as usize;
            let bar = "█".repeat(bar_len);
            println!(
                "\x1b[32m{:<30}\x1b[0m | \x1b[34m{:<10}\x1b[0m | \x1b[33m{:<20}\x1b[0m",
                domain, count, bar
            );
        }
    }
}

struct SafariProvider;

impl SafariProvider {
    const DB_PATH: &'static str = "Library/Safari/History.db";
    const TEMP_DB: &'static str = "/tmp/safari_history_copy";

    fn fetch_history() -> Result<Vec<(String, u32)>> {
        Self::prepare_db_copy()?;

        let conn = Connection::open(Self::TEMP_DB)?;
        let mut stmt = conn.prepare(
            "
            SELECT i.url, COUNT(v.id)
            FROM history_items i
            JOIN history_visits v ON i.id = v.history_item
            WHERE v.visit_time > (strftime('%s', 'now') - 978307200 - 7 * 24 * 3600)
            GROUP BY i.url
            ",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?))
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
        if let Err(e) = fs::copy(src, Self::TEMP_DB) {
            eprintln!("❌ Ошибка копирования базы: {}", e);
            eprintln!("💡 Tip: Check 'Full Disk Access' for your Terminal in System Settings.");
            return Err(rusqlite::Error::InvalidPath(std::path::PathBuf::from(
                Self::TEMP_DB,
            )));
        };

        Ok(())
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let limit = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(15);
    let filter = args.get(2);
    let mut tracker = Tracker::new();

    for (url, count) in SafariProvider::fetch_history()? {
        tracker.process_record(url, count);
    }
    tracker.display(limit, filter);

    Ok(())
}
