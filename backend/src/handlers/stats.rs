use axum::{
    extract::{Query, State},
    response::Response,
};
use serde::Deserialize;

use crate::{
    middleware::AuthenticatedUser,
    models::{StatsPeriod, UserStats},
    services::StatsService,
    state::AppState,
    utils::error::AppError,
    utils::response::success_response,
};

#[derive(Debug, Deserialize)]
pub struct StatsQueryParams {
    pub period: Option<String>,
}

pub async fn get_user_stats(
    State(app_state): State<AppState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Query(query): Query<StatsQueryParams>,
) -> Result<Response, AppError> {
    let period = parse_period(query.period.as_deref())?;
    let stats: UserStats =
        StatsService::get_user_stats(user_id, period, &app_state.db_pool).await?;

    Ok(success_response(stats))
}

fn parse_period(value: Option<&str>) -> Result<StatsPeriod, AppError> {
    match value.unwrap_or("week").to_lowercase().as_str() {
        "week" => Ok(StatsPeriod::Week),
        "month" => Ok(StatsPeriod::Month),
        "year" => Ok(StatsPeriod::Year),
        invalid => Err(AppError::BadRequest(format!(
            "Invalid period '{}', expected week/month/year",
            invalid
        ))),
    }
}
