use crate::provider::VisitRecord;
use std::collections::HashMap;
use std::fs;

pub struct Tracker {
    counts: HashMap<String, u32>,
    hourly_activity: [u32; 24],
}

impl Tracker {
    pub fn new() -> Self {
        Self {
            counts: HashMap::new(),
            hourly_activity: [0; 24],
        }
    }

    pub fn process_record(&mut self, record: VisitRecord) {
        let domain = Self::clean_domain(&record.url);
        *self.counts.entry(domain).or_insert(0) += record.count;

        if record.hour < 24 {
            self.hourly_activity[record.hour] += record.count;
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

    pub fn display(&self, limit: usize, filter: Option<&String>) {
        let mut sorted: Vec<_> = self
            .counts
            .iter()
            .filter(|(domain, _)| filter.map_or(true, |f| domain.contains(f)))
            .collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));

        println!("\n📊 TOP DOMAINS:");
        let max_val = sorted.first().map(|x| *x.1).unwrap_or(1);
        for (domain, count) in sorted.into_iter().take(limit) {
            let bar = "█".repeat((count * 20 / max_val) as usize);
            println!("\x1b[32m{:<30}\x1b[0m | {:<10} | {}", domain, count, bar);
        }

        println!("\n⏰ ACTIVITY BY HOUR:");
        let max_hour = *self.hourly_activity.iter().max().unwrap_or(&1);
        for (h, &c) in self.hourly_activity.iter().enumerate() {
            if c > 0 {
                let bar = "░".repeat((c * 20 / max_hour) as usize);
                println!("{:02}:00 | {:<10} | {}", h, c, bar);
            }
        }
    }

    pub fn export_html(&self, filename: &str) -> std::io::Result<()> {
        let mut sorted: Vec<_> = self.counts.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));

        let labels = sorted
            .iter()
            .take(10)
            .map(|(d, _)| format!("\"{}\"", d))
            .collect::<Vec<_>>()
            .join(",");
        let data = sorted
            .iter()
            .take(10)
            .map(|(_, c)| c.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let hourly_data = self
            .hourly_activity
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(",");

        let template = include_str!("report_template.html");
        let html = template
            .replace("CHART_LABELS", &labels)
            .replace("CHART_DATA", &data)
            .replace("HOURLY_DATA", &hourly_data);

        fs::write(filename, html)
    }
}
