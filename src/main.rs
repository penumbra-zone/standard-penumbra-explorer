mod state;
mod web;

use std::{net::SocketAddr, str::FromStr as _};

pub(crate) use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state =
        AppState::create("postgresql://localhost:5432/penumbra_raw?sslmode=disable").await?;
    let address = SocketAddr::from_str("[::]:1234")?;
    web::WebServer::new(state, address).run().await
}
