use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use serde::{Deserialize, Serialize};

use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestResponse {
    height: u64,
}

async fn fetch_height(state: &AppState) -> anyhow::Result<u64> {
    let (height,): (i64,) = sqlx::query_as("SELECT max(height) FROM blocks;")
        .fetch_one(state.pool())
        .await?;
    let height = u64::try_from(height)?;
    Ok(height)
}

async fn handler(State(state): State<AppState>) -> Result<Json<TestResponse>, StatusCode> {
    let height = fetch_height(&state)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(TestResponse { height }))
}

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(handler))
}
