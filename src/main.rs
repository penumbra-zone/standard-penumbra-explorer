mod component;
mod error;
mod indexer;
pub(self) mod state;
mod web;

use std::{io::IsTerminal as _, net::SocketAddr, str::FromStr as _};

use tracing_subscriber::EnvFilter;

use crate::state::AppState;

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_ansi(std::io::stdout().is_terminal())
        .with_env_filter(
            EnvFilter::from_default_env()
                // Without explicitly disabling the `r1cs` target, the ZK proof implementations
                // will spend an enormous amount of CPU and memory building useless tracing output.
                .add_directive(
                    "r1cs=off"
                        .parse()
                        .expect("rics=off is a valid filter directive"),
                ),
        )
        .with_writer(std::io::stderr)
        .init();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let state = AppState::create("postgresql://localhost:5432/penumbra?sslmode=disable").await?;
    let address = SocketAddr::from_str("[::]:1234")?;
    let web_server_handle = tokio::spawn(web::WebServer::new(state, address).run());
    let indexer_handle = indexer::Indexer::new().run();

    tokio::select! {
        x = web_server_handle => x?,
        x = indexer_handle => x
    }
}
