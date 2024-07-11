use std::path::Path;

use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use include_dir::{include_dir, Dir};

use crate::state::AppState;

const STATIC_FILES: Dir = include_dir!("static");

/// A type for responses which serve static content, with a given type.
#[derive(Clone)]
struct ContentResponse {
    content_type: &'static str,
    data: &'static [u8],
}

impl<'data> IntoResponse for ContentResponse {
    fn into_response(self) -> axum::response::Response<Body> {
        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", self.content_type)
            .header("Cache-Control", "public, max-age=3600")
            .body(self.data.into())
            .expect("content response should be valid")
    }
}

/// Include a static file into the router.
///
/// The path should include the folder where static files reside.
/// For example, if the file is in /static/foo.txt, then we'll serve that file at /static/foo.txt
fn route_file(router: Router<AppState>, path: &Path, data: &'static [u8]) -> Router<AppState> {
    let content_type: &'static str = match path.extension().and_then(|x| x.to_str()) {
        Some("css") => "text/css",
        Some("woff2") => "font/woff2",
        _ => "application/octet-stream",
    };
    let routing_path = format!(
        "/{}",
        path.to_str()
            .expect("static file path should be a valid str")
    );
    let resp = ContentResponse { content_type, data };

    router.route(routing_path.as_str(), get(move || async { resp }))
}

pub fn router() -> Router<AppState> {
    // Add all the files into the router
    let mut router = Router::new();
    for file in STATIC_FILES.files() {
        //       tracing::info!(contents = String::from_utf8_lossy(file.contents()).to_string());
        router = route_file(router, file.path(), file.contents());
    }
    router
}
