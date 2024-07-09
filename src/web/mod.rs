mod blocks;
mod example;
mod index;

use axum::{
    extract::{MatchedPath, Request},
    Router,
};
use core::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing::info_span;

use crate::state::AppState;

/// Represents the configuration of the web server.
///
/// This is the entry point to the frontend, and running it will serve the web pages.
pub struct WebServer {
    address: SocketAddr,
    state: AppState,
}

impl WebServer {
    pub fn new(state: AppState, address: SocketAddr) -> Self {
        Self { state, address }
    }

    #[allow(dead_code)]
    pub fn with_address(mut self, addr: SocketAddr) -> Self {
        self.address = addr;
        self
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let app = Router::new()
            .nest("/", index::router())
            .nest("/example", example::router())
            .nest("/history/blocks", blocks::router())
            .with_state(self.state)
            .layer(
                TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                    // Log the matched route's path (with placeholders not filled in).
                    // Use request.uri() or OriginalUri if you want the real path.
                    let map = request
                        .extensions()
                        .get::<MatchedPath>()
                        .map(MatchedPath::as_str);
                    let matched_path = map;

                    info_span!(
                        "http",
                        method = ?request.method(),
                        matched_path,
                    )
                }),
            );

        let listener = tokio::net::TcpListener::bind(self.address).await?;
        axum::serve(listener, app).await?;
        Ok(())
    }
}
