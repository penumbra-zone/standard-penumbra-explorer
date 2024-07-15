use axum::response::{Html, IntoResponse, Response};
use axum::{extract::State, routing::get, Json, Router};
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::state::AppState;

use super::common::AcceptsJson;

use crate::component::block::{Block, Component};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BlocksResponse {
    blocks: Vec<Block>,
}

async fn handler(
    State(state): State<AppState>,
    AcceptsJson(json): AcceptsJson,
) -> Result<Response> {
    let resp = BlocksResponse {
        blocks: Component::blocks(state.pool(), 100).await?,
    };

    if json {
        Ok(Json(resp).into_response())
    } else {
        Ok(Html(state.render_template(Component::TEMPLATE.0, resp)?).into_response())
    }
}

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(handler))
}
