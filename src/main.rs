use axum::{
    routing::{get, MethodRouter},
    Json, Router,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestResponse {
    message: String,
}

fn test() -> MethodRouter {
    get(|| async {
        Json(TestResponse {
            message: "foo".to_string(),
        })
    })
}

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/test.json", test());

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
