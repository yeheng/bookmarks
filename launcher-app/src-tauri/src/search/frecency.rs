use crate::error::AppResult;
use crate::services::data_service::DataService;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Decay factor for frecency score.
/// Score = Initial * DECAY_FACTOR ^ (days_since_usage)
/// 0.9 means score drops to ~50% after 7 days.
const DECAY_FACTOR: f64 = 0.9;
const INITIAL_SCORE: f64 = 100.0;

/// In-memory usage log entry.
struct UsageEvent {
    accessed_at: i64,
}

pub struct FrecencyTracker {
    data_service: Arc<DataService>,
    /// In-memory usage log: (source_id, item_id) -> events
    usage_log: Mutex<HashMap<(String, String), Vec<UsageEvent>>>,
}

impl FrecencyTracker {
    pub fn new(data_service: Arc<DataService>) -> Self {
        Self {
            data_service,
            usage_log: Mutex::new(HashMap::new()),
        }
    }

    /// Record a usage event (e.g. user clicked a result)
    pub fn record_usage(&self, source_id: &str, item_id: &str) -> AppResult<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        if let Ok(mut log) = self.usage_log.lock() {
            let key = (source_id.to_string(), item_id.to_string());
            log.entry(key)
                .or_default()
                .push(UsageEvent { accessed_at: now });
        }

        Ok(())
    }

    /// Calculate the frecency score for an item.
    /// Uses in-memory usage events and applies decay.
    pub fn get_score(&self, source_id: &str, item_id: &str) -> AppResult<f64> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        const DAY_SECS: i64 = 86400;

        let log = self.usage_log.lock().map_err(|_| crate::error::AppError::StoreLock)?;
        let key = (source_id.to_string(), item_id.to_string());

        let total_score = match log.get(&key) {
            Some(events) => {
                let mut score = 0.0;
                for event in events {
                    let age_secs = (now - event.accessed_at).max(0);
                    let age_days = age_secs as f64 / DAY_SECS as f64;
                    score += INITIAL_SCORE * DECAY_FACTOR.powf(age_days);
                }
                score
            }
            None => 0.0,
        };

        Ok(total_score)
    }
}
