use rusqlite::{Connection, Result};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

struct Tracker {
    counts: HashMap<String, u32>,
}

impl Tracker {
    fn new() -> Self {
        Tracker {
            counts: HashMap::new(),
        }
    }

    fn clean_domain(url: &str) -> String {
        url.split('/')
            .nth(2)
            .unwrap_or(url)
            .trim_start_matches('.')
            .replace("www.", "")
            .to_lowercase()
    }

    fn process_record(&mut self, url: String, count: u32) {
        let domain = Self::clean_domain(&url);
        *self.counts.entry(domain).or_insert(0) += count;
    }

    fn display(&self, limit: usize, filter: Option<&String>) {
        let mut sorted: Vec<_> = self
            .counts
            .iter()
            .filter(|(domain, _)| filter.map_or(true, |f| domain.contains(f)))
            .collect();

        sorted.sort_by(|a, b| b.1.cmp(a.1));

        if sorted.is_empty() {
            println!("\nНичего не найдено.");
            return;
        }

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

    fn export_html(&self, filename: &str) -> std::io::Result<()> {
        let mut sorted: Vec<_> = self.counts.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));
        let top_10: Vec<_> = sorted.into_iter().take(10).collect();

        let labels_str = top_10
            .iter()
            .map(|(d, _)| format!("\"{}\"", d))
            .collect::<Vec<String>>()
            .join(",");

        let data_str = top_10
            .iter()
            .map(|(_, c)| c.to_string())
            .collect::<Vec<String>>()
            .join(",");

        let template = include_str!("report_template.html");

        let html = template
            .replace("CHART_LABELS", &labels_str)
            .replace("CHART_DATA", &data_str);

        fs::write(filename, html)?;
        println!("\n✅ Отчет готов: {}", filename);
        Ok(())
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
            // Исправлено: передаем индекс (usize) и явно указываем тип для десериализации
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

fn main() -> Result<()> {
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
