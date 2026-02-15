use crate::error::AppResult;
use crate::services::data_service::DataService;
use rusqlite::params;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Decay factor for frecency score.
/// Score = Initial * DECAY_FACTOR ^ (days_since_usage)
/// 0.9 means score drops to ~50% after 7 days.
const DECAY_FACTOR: f64 = 0.9;
const INITIAL_SCORE: f64 = 100.0;

pub struct FrecencyTracker {
    data_service: Arc<DataService>,
}

impl FrecencyTracker {
    pub fn new(data_service: Arc<DataService>) -> Self {
        Self { data_service }
    }

    /// Record a usage event (e.g. user clicked a result)
    pub fn record_usage(&self, source_id: &str, item_id: &str) -> AppResult<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        self.data_service.with_db(|conn| {
            conn.execute(
                "INSERT INTO usage_log (source_id, item_id, accessed_at) VALUES (?1, ?2, ?3)",
                params![source_id, item_id, now],
            )?;
            Ok(())
        })
    }

    /// Calculate the frecency score for an item.
    /// Fetches all usage events, applies decay, and sums them up.
    pub fn get_score(&self, source_id: &str, item_id: &str) -> AppResult<f64> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        // One day in seconds
        const DAY_SECS: i64 = 86400;

        self.data_service.with_db(|conn| {
            let mut stmt = conn.prepare(
                "SELECT accessed_at FROM usage_log WHERE source_id = ?1 AND item_id = ?2",
            )?;

            let timestamps =
                stmt.query_map(params![source_id, item_id], |row| row.get::<_, i64>(0))?;

            let mut total_score = 0.0;

            for ts in timestamps {
                let ts = ts?;
                let age_secs = (now - ts).max(0);
                let age_days = age_secs as f64 / DAY_SECS as f64;

                // Score = 100 * 0.9 ^ days
                let score = INITIAL_SCORE * DECAY_FACTOR.powf(age_days);
                total_score += score;
            }

            Ok(total_score)
        })
    }
}
