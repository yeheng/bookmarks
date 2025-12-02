use chrono::{Duration, NaiveDate, Utc};
use sqlx::{SqlitePool, Row};
use uuid::Uuid;

use crate::models::{RecentActivityEntry, StatsPeriod, TopDomainEntry, TopTagEntry, UserStats};
use crate::utils::error::{AppError, AppResult};

pub struct StatsService;

impl StatsService {
    pub async fn get_user_stats(
        user_id: Uuid,
        period: StatsPeriod,
        db_pool: &SqlitePool,
    ) -> AppResult<UserStats> {
        let summary = sqlx::query(
            r#"
            SELECT
                COUNT(*) as total_bookmarks,
                SUM(CASE WHEN is_favorite = 1 THEN 1 ELSE 0 END) as favorite_bookmarks,
                SUM(CASE WHEN is_archived = 1 THEN 1 ELSE 0 END) as archived_bookmarks,
                SUM(CASE WHEN is_private = 1 THEN 1 ELSE 0 END) as private_bookmarks,
                SUM(CASE WHEN is_read = 1 THEN 1 ELSE 0 END) as read_bookmarks,
                SUM(visit_count) as total_visits
            FROM bookmarks
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(db_pool)
        .await?
        .ok_or_else(|| AppError::Internal("Failed to gather summary stats".to_string()))?;

        let start_date = Self::start_date(period);
        let recent_activity = Self::recent_activity(user_id, start_date, db_pool).await?;
        let top_tags = Self::top_tags(user_id, db_pool).await?;
        let top_domains = Self::top_domains(user_id, db_pool).await?;

        let total_bookmarks = summary
            .get::<Option<i64>, _>("total_bookmarks")
            .unwrap_or(0);
        let total_collections = summary
            .get::<Option<i64>, _>("total_collections")
            .unwrap_or(0);
        let total_tags = summary.get::<Option<i64>, _>("total_tags").unwrap_or(0);
        let favorite_bookmarks = summary
            .get::<Option<i64>, _>("favorite_bookmarks")
            .unwrap_or(0);
        let archived_bookmarks = summary
            .get::<Option<i64>, _>("archived_bookmarks")
            .unwrap_or(0);
        let total_visits = summary.get::<Option<i64>, _>("total_visits").unwrap_or(0);

        Ok(UserStats {
            total_bookmarks,
            total_collections,
            total_tags,
            favorite_bookmarks,
            archived_bookmarks,
            total_visits,
            recent_activity,
            top_tags,
            top_domains,
        })
    }

    async fn recent_activity(
        user_id: Uuid,
        start_date: NaiveDate,
        db_pool: &SqlitePool,
    ) -> AppResult<Vec<RecentActivityEntry>> {
        // SQLite doesn't have generate_series, so we'll get the actual data and fill gaps in code
        let rows = sqlx::query(
            r#"
            SELECT DATE(created_at) AS activity_date, COUNT(*) AS bookmarks_added
            FROM bookmarks
            WHERE user_id = $1 AND DATE(created_at) >= DATE($2)
            GROUP BY DATE(created_at)
            
            UNION ALL
            
            SELECT DATE(last_visited) AS activity_date, COUNT(*) AS bookmarks_visited
            FROM bookmarks
            WHERE user_id = $1 AND last_visited IS NOT NULL AND DATE(last_visited) >= DATE($2)
            GROUP BY DATE(last_visited)
            ORDER BY activity_date DESC
            LIMIT 60
            "#,
        )
        .bind(user_id)
        .bind(start_date)
        .fetch_all(db_pool)
        .await?;

        {
            let mut activities = std::collections::HashMap::new();
            
            // Process the rows and aggregate by date
            for row in rows {
                let date: NaiveDate = row.get("activity_date");
                let bookmarks_added: Option<i64> = row.get("bookmarks_added");
                let bookmarks_visited: Option<i64> = row.get("bookmarks_visited");
                
                let entry = activities.entry(date).or_insert(RecentActivityEntry {
                    date,
                    bookmarks_added: 0,
                    bookmarks_visited: 0,
                });
                
                if bookmarks_added.is_some() {
                    entry.bookmarks_added += bookmarks_added.unwrap_or(0);
                } else if bookmarks_visited.is_some() {
                    entry.bookmarks_visited += bookmarks_visited.unwrap_or(0);
                }
            }
            
            // Convert to sorted vector
            let mut result: Vec<RecentActivityEntry> = activities.into_values().collect();
            result.sort_by(|a, b| b.date.cmp(&a.date));
            result.truncate(30);
            Ok(result)
        }
    }

    async fn top_tags(user_id: Uuid, db_pool: &SqlitePool) -> AppResult<Vec<TopTagEntry>> {
        let rows = sqlx::query(
            r#"
            SELECT t.name, COUNT(bt.bookmark_id) AS usage_count
            FROM tags t
            LEFT JOIN bookmark_tags bt ON t.id = bt.tag_id
            WHERE t.user_id = $1
            GROUP BY t.name
            ORDER BY usage_count DESC
            LIMIT 5
            "#,
        )
        .bind(user_id)
        .fetch_all(db_pool)
        .await?;

        Ok(rows
            .into_iter()
            .filter_map(|row| {
                let name: Option<String> = row.get("name");
                name.map(|name| TopTagEntry {
                    name,
                    count: row.get::<Option<i64>, _>("usage_count").unwrap_or(0),
                })
            })
            .collect())
    }

    async fn top_domains(user_id: Uuid, db_pool: &SqlitePool) -> AppResult<Vec<TopDomainEntry>> {
        let rows = sqlx::query(
            r#"
            SELECT domain, COUNT(*) AS domain_count
            FROM (
                SELECT substr(url, instr(url, '://') + 3, instr(substr(url, instr(url, '://') + 3), '/') - 1) AS domain
                FROM bookmarks
                WHERE user_id = $1
            ) d
            WHERE domain IS NOT NULL AND domain <> ''
            GROUP BY domain
            ORDER BY domain_count DESC
            LIMIT 5
            "#,
        )
        .bind(user_id)
        .fetch_all(db_pool)
        .await?;

        Ok(rows
            .into_iter()
            .filter_map(|row| {
                let domain: Option<String> = row.get("domain");
                domain.map(|domain| TopDomainEntry {
                    domain,
                    count: row.get::<Option<i64>, _>("domain_count").unwrap_or(0),
                })
            })
            .collect())
    }

    fn start_date(period: StatsPeriod) -> NaiveDate {
        let today = Utc::now().date_naive();
        let offset = period.days().saturating_sub(1);
        today - Duration::days(offset)
    }
}
