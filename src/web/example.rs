use axum::{routing::get, Json, Router};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestResponse {
    message: String,
}

fn example() -> TestResponse {
    TestResponse {
        message: "EXAMPLE".to_string(),
    }
}

pub fn router() -> Router {
    Router::new().route("/", get(|| async { Json(example()) }))
}
