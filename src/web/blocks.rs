use axum::{extract::State, routing::get, Json, Router};
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BlocksResponse {
    blocks: Vec<crate::component::block::Block>,
}

async fn handler(State(state): State<AppState>) -> Result<Json<BlocksResponse>> {
    let blocks = crate::component::block::Component::blocks(state.pool(), 20).await?;
    Ok(Json(BlocksResponse { blocks }))
}

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(handler))
}
