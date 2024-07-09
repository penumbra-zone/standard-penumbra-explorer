mod example;
mod index;

use axum::Router;
use core::net::SocketAddr;

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
            .with_state(self.state);
        let listener = tokio::net::TcpListener::bind(self.address).await?;
        axum::serve(listener, app).await?;
        Ok(())
    }
}
