use chrono::{Duration, NaiveDate, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::models::{RecentActivityEntry, StatsPeriod, TopDomainEntry, TopTagEntry, UserStats};
use crate::utils::error::{AppError, AppResult};

pub struct StatsService;

impl StatsService {
    pub async fn get_user_stats(
        user_id: Uuid,
        period: StatsPeriod,
        db_pool: &PgPool,
    ) -> AppResult<UserStats> {
        let summary = sqlx::query(
            r#"
            SELECT
                COUNT(*)::bigint as total_bookmarks,
                COUNT(*) FILTER (WHERE is_favorite = TRUE)::bigint as favorite_bookmarks,
                COUNT(*) FILTER (WHERE is_archived = TRUE)::bigint as archived_bookmarks,
                COALESCE(SUM(visit_count), 0)::bigint as total_visits,
                (SELECT COUNT(*)::bigint FROM collections WHERE user_id = $1) as total_collections,
                (SELECT COUNT(*)::bigint FROM tags WHERE user_id = $1) as total_tags
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
        db_pool: &PgPool,
    ) -> AppResult<Vec<RecentActivityEntry>> {
        let rows = sqlx::query(
            r#"
            WITH date_series AS (
                SELECT generate_series($2::date, CURRENT_DATE, '1 day')::date AS activity_date
            ),
            added AS (
                SELECT DATE(created_at) AS activity_date, COUNT(*)::bigint AS bookmarks_added
                FROM bookmarks
                WHERE user_id = $1 AND created_at::date >= $2
                GROUP BY DATE(created_at)
            ),
            visited AS (
                SELECT DATE(last_visited) AS activity_date, COUNT(*)::bigint AS bookmarks_visited
                FROM bookmarks
                WHERE user_id = $1 AND last_visited IS NOT NULL AND last_visited::date >= $2
                GROUP BY DATE(last_visited)
            )
            SELECT ds.activity_date,
                   COALESCE(a.bookmarks_added, 0) AS bookmarks_added,
                   COALESCE(v.bookmarks_visited, 0) AS bookmarks_visited
            FROM date_series ds
            LEFT JOIN added a ON a.activity_date = ds.activity_date
            LEFT JOIN visited v ON v.activity_date = ds.activity_date
            ORDER BY ds.activity_date DESC
            LIMIT 30
            "#,
        )
        .bind(user_id)
        .bind(start_date)
        .fetch_all(db_pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| RecentActivityEntry {
                date: row
                    .get::<Option<NaiveDate>, _>("activity_date")
                    .unwrap_or(start_date),
                bookmarks_added: row.get::<Option<i64>, _>("bookmarks_added").unwrap_or(0),
                bookmarks_visited: row.get::<Option<i64>, _>("bookmarks_visited").unwrap_or(0),
            })
            .collect())
    }

    async fn top_tags(user_id: Uuid, db_pool: &PgPool) -> AppResult<Vec<TopTagEntry>> {
        let rows = sqlx::query(
            r#"
            SELECT t.name, COUNT(bt.bookmark_id)::bigint AS usage_count
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

    async fn top_domains(user_id: Uuid, db_pool: &PgPool) -> AppResult<Vec<TopDomainEntry>> {
        let rows = sqlx::query(
            r#"
            SELECT domain, COUNT(*)::bigint AS domain_count
            FROM (
                SELECT regexp_replace(url, '^https?://([^/]+).*', '\1') AS domain
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
