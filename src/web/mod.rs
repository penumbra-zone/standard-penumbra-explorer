mod example;
mod index;

use axum::Router;
use core::net::SocketAddr;
use core::str::FromStr;

/// Represents the configuration of the web server.
///
/// This is the entry point to the frontend, and running it will serve the web pages.
#[derive(Clone)]
pub struct WebServer {
    address: SocketAddr,
}

impl WebServer {
    pub fn new() -> Self {
        Self {
            address: SocketAddr::from_str("[::]:1234").unwrap(),
        }
    }

    #[allow(dead_code)]
    pub fn with_address(mut self, addr: SocketAddr) -> Self {
        self.address = addr;
        self
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let app = Router::new()
            .nest("/", index::router())
            .nest("/example", example::router());
        let listener = tokio::net::TcpListener::bind(self.address).await?;
        axum::serve(listener, app).await?;
        Ok(())
    }
}
