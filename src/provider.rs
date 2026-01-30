use anyhow::{Context, Result};
use rusqlite::Connection;
use std::env;
use std::fs;
use std::path::Path;
use tracing::{debug, warn};

pub struct SafariProvider;

impl SafariProvider {
    const DB_PATH: &'static str = "Library/Safari/History.db";
    const TEMP_DB: &'static str = "/tmp/safari_history_copy";

    pub fn fetch_history(days: u32) -> Result<Vec<(String, u32)>> {
        debug!("Начинаю сбор истории за последние {} дней", days);
        Self::prepare_db_copy().context("Ошибка при подготовке копии базы данных")?;

        let conn =
            Connection::open(Self::TEMP_DB).context("Не удалось открыть Sqlite соединение")?;
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
        let home = env::var("HOME").context("Переменная окружения HOME не задана")?;
        let src = Path::new(&home).join(Self::DB_PATH);

        debug!("Копируем базу из {:?} в {}", src, Self::TEMP_DB);

        if let Err(e) = fs::copy(&src, Self::TEMP_DB) {
            warn!("Файл базы не найден или нет прав. Проверьте Full Disk Access.");
            return Err(anyhow::anyhow!("fs::copy failed: {}", e));
        }

        Ok(())
    }
}
