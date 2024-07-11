use axum::{routing::get, Router};
use include_dir::{include_dir, Dir};

use crate::state::AppState;

const STATIC_FILES: Dir = include_dir!("static");

pub fn router() -> Router<AppState> {
    // Add all the files into the router
    let mut router = Router::new();
    for file in STATIC_FILES.files() {
        router = router.route(
            format!(
                "/{}",
                file.path()
                    .to_str()
                    .expect("static file path should be a valid str")
            )
            .as_str(),
            get(move || async { file.contents() }),
        );
    }
    router
}
