use axum::response::{Html, IntoResponse, Response};
use axum::{extract::State, routing::get, Json, Router};
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::state::AppState;

use super::common::AcceptsJson;

use crate::component::validator::{Component, Validator};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ValidatorsResponse {
    validators: Vec<Validator>,
}

async fn handler(
    State(state): State<AppState>,
    AcceptsJson(json): AcceptsJson,
) -> Result<Response> {
    let resp = ValidatorsResponse {
        validators: Component::validators(state.pool()).await?,
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
