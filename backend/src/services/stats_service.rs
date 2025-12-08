use chrono::Utc;
use sqlx::{Row, SqlitePool};

use crate::models::{
    RecentActivityEntry, ResourceWithTags, StatsPeriod, TopDomainEntry, TopTagEntry, UserStats,
};
use crate::utils::error::{AppError, AppResult};

pub struct StatsService;

impl StatsService {
    pub async fn get_user_stats(
        user_id: i64,
        period: StatsPeriod,
        db_pool: &SqlitePool,
    ) -> AppResult<UserStats> {
        // Get resources statistics
        let resource_summary = sqlx::query(
            r#"
            SELECT
                COUNT(*) as total_resources,
                SUM(CASE WHEN is_favorite = 1 THEN 1 ELSE 0 END) as favorite_resources,
                SUM(CASE WHEN is_archived = 1 THEN 1 ELSE 0 END) as archived_resources,
                SUM(CASE WHEN is_private = 1 THEN 1 ELSE 0 END) as private_resources,
                SUM(CASE WHEN is_read = 1 THEN 1 ELSE 0 END) as read_resources,
                SUM(COALESCE(visit_count, 0)) as total_visits
            FROM resources
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(db_pool)
        .await?
        .ok_or_else(|| {
            AppError::Internal("Failed to gather resources stats: no data returned".to_string())
        })?;

        // Get collections count
        let total_collections =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM collections WHERE user_id = $1")
                .bind(user_id)
                .fetch_one(db_pool)
                .await?;

        // Get tags count
        let total_tags =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM tags WHERE user_id = $1")
                .bind(user_id)
                .fetch_one(db_pool)
                .await?;

        let start_date = Self::start_date(period);
        let recent_resources = Self::recent_resources(user_id, db_pool).await?;
        let recent_activity = Self::recent_activity(user_id, start_date, db_pool).await?;
        let top_tags = Self::top_tags(user_id, db_pool).await?;
        let top_domains = Self::top_domains(user_id, db_pool).await?;

        let total_resources = resource_summary
            .get::<Option<i64>, _>("total_resources")
            .unwrap_or(0);
        let favorite_resources = resource_summary
            .get::<Option<i64>, _>("favorite_resources")
            .unwrap_or(0);
        let archived_resources = resource_summary
            .get::<Option<i64>, _>("archived_resources")
            .unwrap_or(0);
        let total_visits = resource_summary
            .get::<Option<i64>, _>("total_visits")
            .unwrap_or(0);

        Ok(UserStats {
            total_resources,
            total_collections,
            total_tags,
            favorite_resources,
            archived_resources,
            total_visits,
            recent_resources,
            recent_activity,
            top_tags,
            top_domains,
        })
    }

    // 查询最近添加的资源 (最多10条)
    async fn recent_resources(
        user_id: i64,
        db_pool: &SqlitePool,
    ) -> AppResult<Vec<ResourceWithTags>> {
        let resources = sqlx::query_as::<_, ResourceWithTags>(
            r#"
            SELECT
                r.*,
                COALESCE(
                    json_group_array(
                        CASE WHEN t.name IS NOT NULL THEN t.name ELSE NULL END
                    ) FILTER (WHERE t.name IS NOT NULL),
                    '[]'
                ) as tags,
                c.name as collection_name,
                c.color as collection_color
            FROM resources r
            LEFT JOIN resource_tags rt ON r.id = rt.resource_id
            LEFT JOIN tags t ON rt.tag_id = t.id
            LEFT JOIN collections c ON r.collection_id = c.id
            WHERE r.user_id = $1
            GROUP BY r.id
            ORDER BY r.created_at DESC
            LIMIT 10
            "#,
        )
        .bind(user_id)
        .fetch_all(db_pool)
        .await?;

        Ok(resources)
    }

    async fn recent_activity(
        user_id: i64,
        start_date: i64,
        db_pool: &SqlitePool,
    ) -> AppResult<Vec<RecentActivityEntry>> {
        // Get recent activity by directly working with timestamps
        let rows = sqlx::query(
            r#"
            SELECT 
                (created_at / 86400) * 86400 as day_timestamp,
                COUNT(*) as resources_added,
                0 as resources_visited
            FROM resources
            WHERE user_id = $1 AND created_at >= $2
            GROUP BY (created_at / 86400)

            UNION ALL

            SELECT 
                (last_visited / 86400) * 86400 as day_timestamp,
                0 as resources_added,
                COUNT(*) as resources_visited
            FROM resources
            WHERE user_id = $1 AND last_visited IS NOT NULL AND last_visited >= $2
            GROUP BY (last_visited / 86400)
            ORDER BY day_timestamp DESC
            LIMIT 60
            "#,
        )
        .bind(user_id)
        .bind(start_date)
        .fetch_all(db_pool)
        .await?;

        let mut activities = std::collections::HashMap::new();

        // Process the rows and aggregate by day
        for row in rows {
            let day_timestamp: i64 = row.get("day_timestamp");
            let resources_added: i64 = row.get("resources_added");
            let resources_visited: i64 = row.get("resources_visited");

            let entry = activities
                .entry(day_timestamp)
                .or_insert(RecentActivityEntry {
                    date: day_timestamp,
                    resources_added: 0,
                    resources_visited: 0,
                });

            entry.resources_added += resources_added;
            entry.resources_visited += resources_visited;
        }

        // Convert to sorted vector
        let mut result: Vec<RecentActivityEntry> = activities.into_values().collect();
        result.sort_by(|a, b| b.date.cmp(&a.date));
        result.truncate(30);
        Ok(result)
    }

    async fn top_tags(user_id: i64, db_pool: &SqlitePool) -> AppResult<Vec<TopTagEntry>> {
        let rows = sqlx::query(
            r#"
            SELECT t.name, COUNT(rt.resource_id) AS usage_count
            FROM tags t
            LEFT JOIN resource_tags rt ON t.id = rt.tag_id
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

    async fn top_domains(user_id: i64, db_pool: &SqlitePool) -> AppResult<Vec<TopDomainEntry>> {
        let rows = sqlx::query(
            r#"
            SELECT domain, COUNT(*) AS domain_count
            FROM (
                SELECT substr(url, instr(url, '://') + 3, instr(substr(url, instr(url, '://') + 3), '/') - 1) AS domain
                FROM resources
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

    fn start_date(period: StatsPeriod) -> i64 {
        let today = Utc::now().timestamp();
        let offset = period.days().saturating_sub(1);
        today - offset
    }
}
