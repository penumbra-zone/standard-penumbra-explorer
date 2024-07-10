use axum::response::{Html, IntoResponse, Response};
use axum::{extract::State, routing::get, Json, Router};
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::state::AppState;

use super::common::AcceptsJson;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BlocksResponse {
    blocks: Vec<crate::component::block::Block>,
}

async fn handler(
    State(state): State<AppState>,
    AcceptsJson(json): AcceptsJson,
) -> Result<Response> {
    let blocks = crate::component::block::Component::blocks(state.pool(), 20).await?;
    if json {
        Ok(Json(BlocksResponse { blocks }).into_response())
    } else {
        Ok(Html(serde_json::to_string(&BlocksResponse { blocks })?).into_response())
    }
}

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(handler))
}
