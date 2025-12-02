use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum StatsPeriod {
    #[default]
    Week,
    Month,
    Year,
}

impl StatsPeriod {
    pub fn default_days() -> i64 {
        Self::Week.days()
    }

    pub fn days(self) -> i64 {
        match self {
            StatsPeriod::Week => 7,
            StatsPeriod::Month => 30,
            StatsPeriod::Year => 365,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct UserStats {
    pub total_bookmarks: i64,
    pub total_collections: i64,
    pub total_tags: i64,
    pub favorite_bookmarks: i64,
    pub archived_bookmarks: i64,
    pub total_visits: i64,
    pub recent_activity: Vec<RecentActivityEntry>,
    pub top_tags: Vec<TopTagEntry>,
    pub top_domains: Vec<TopDomainEntry>,
}

#[derive(Debug, Serialize)]
pub struct RecentActivityEntry {
    pub date: NaiveDate,
    pub bookmarks_added: i64,
    pub bookmarks_visited: i64,
}

#[derive(Debug, Serialize)]
pub struct TopTagEntry {
    pub name: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct TopDomainEntry {
    pub domain: String,
    pub count: i64,
}
