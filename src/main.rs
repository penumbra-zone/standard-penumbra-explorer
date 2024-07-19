mod component;
mod error;
mod indexer;
mod pagination;
mod sql;
pub(self) mod state;
mod web;

use std::{io::IsTerminal as _, net::SocketAddr, str::FromStr as _};

use clap::Parser;
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

#[derive(Clone, Debug, Parser)]
struct Options {
    /// The listening address, if the web server should be run
    #[clap(long)]
    web: Option<String>,
    #[clap(flatten)]
    indexer: pindexer::Options,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let opt = Options::parse();

    let state = AppState::create(opt.indexer.dst_database_url.as_ref()).await?;
    let web_server_handle = if let Some(web) = opt.web {
        let address = SocketAddr::from_str(web.as_ref())?;
        tokio::spawn(web::WebServer::new(state, address).run())
    } else {
        tokio::spawn(async {
            loop {
                tokio::task::yield_now().await
            }
        })
    };

    let indexer_handle = indexer::Indexer::new(opt.indexer).run();

    tokio::select! {
        x = web_server_handle => x?,
        x = indexer_handle => x
    }
}
