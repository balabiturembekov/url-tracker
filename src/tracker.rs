use std::collections::HashMap;
use std::fs;

pub struct Tracker {
    counts: HashMap<String, u32>,
}

impl Tracker {
    pub fn new() -> Self {
        Tracker {
            counts: HashMap::new(),
        }
    }

    pub fn clean_domain(url: &str) -> String {
        url.split('/')
            .nth(2)
            .unwrap_or(url)
            .trim_start_matches('.')
            .replace("www.", "")
            .to_lowercase()
    }

    pub fn process_record(&mut self, url: String, count: u32) {
        let domain = Self::clean_domain(&url);
        *self.counts.entry(domain).or_insert(0) += count;
    }

    pub fn display(&self, limit: usize, filter: Option<&String>) {
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

    pub fn export_html(&self, filename: &str) -> std::io::Result<()> {
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
