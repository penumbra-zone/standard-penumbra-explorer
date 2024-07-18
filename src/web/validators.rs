use axum::extract::Path;
use axum::response::{Html, IntoResponse, Response};
use axum::{extract::State, routing::get, Json, Router};
use penumbra_stake::IdentityKey;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use crate::error::Result;
use crate::state::AppState;

use super::common::AcceptsJson;

use crate::component::validator::{Component, Validator, ValidatorSummary};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ValidatorsResponse {
    validators: Vec<ValidatorSummary>,
}

async fn all_handler(
    State(state): State<AppState>,
    AcceptsJson(json): AcceptsJson,
) -> Result<Response> {
    let resp = ValidatorsResponse {
        validators: Component::validators(state.pool()).await?,
    };

    if json {
        Ok(Json(resp).into_response())
    } else {
        Ok(Html(state.render_template(Component::TEMPLATES[0].0, resp)?).into_response())
    }
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
struct IdentityKeyString {
    #[serde_as(as = "DisplayFromStr")]
    ik: IdentityKey,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ValidatorResponse {
    validator: Validator,
}

async fn single_handler(
    State(state): State<AppState>,
    AcceptsJson(json): AcceptsJson,
    Path(IdentityKeyString { ik }): Path<IdentityKeyString>,
) -> Result<Response> {
    let resp = ValidatorResponse {
        validator: Component::validator(state.pool(), &ik).await?,
    };

    if json {
        Ok(Json(resp).into_response())
    } else {
        Ok(Html(state.render_template(Component::TEMPLATES[1].0, resp)?).into_response())
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(all_handler))
        .route("/:ik", get(single_handler))
}
