use serde::{Deserialize, Serialize};

use super::ResourceWithTags;

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
    #[allow(dead_code)]
    pub fn default_days() -> i64 {
        Self::Week.days()
    }

    pub fn days(self) -> i64 {
        match self {
            StatsPeriod::Week => 7 * 24 * 60 * 60,
            StatsPeriod::Month => 30 * 24 * 60 * 60,
            StatsPeriod::Year => 365 * 24 * 60 * 60,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct UserStats {
    pub total_resources: i64,
    pub total_collections: i64,
    pub total_tags: i64,
    pub favorite_resources: i64,
    pub archived_resources: i64,
    pub total_visits: i64,
    pub recent_resources: Vec<ResourceWithTags>, // 最近资源列表
    pub recent_activity: Vec<RecentActivityEntry>,
    pub top_tags: Vec<TopTagEntry>,
    pub top_domains: Vec<TopDomainEntry>,
}

#[derive(Debug, Serialize)]
pub struct RecentActivityEntry {
    pub date: i64,
    pub resources_added: i64,
    pub resources_visited: i64,
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
