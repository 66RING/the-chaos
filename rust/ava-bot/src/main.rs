use anyhow::Result;
use axum::routing::{get, post};
use axum::Router;
use axum_server::tls_rustls::RustlsConfig;
use tower_http::services::ServeDir;
use clap::Parser;
use std::sync::Arc;
use tracing::info;

use ava_bot::handlers::*;

#[derive(Parser, Debug)]
#[clap(name = "ava")]
struct Args {
    /// Port to listen on
    #[clap(short, long, default_value = "8080")]
    port: u16,
    /// Path to cert file
    #[clap(short, long, default_value = "./.certs")]
    cert: String,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct AppState {}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let state = Arc::new(AppState::default);

    let app = Router::new()
        .route("/", get(index_page))
        .route("/chats", get(chats_handler))
        .route("/assistant", post(assistant_handler))
        // Serve static file.
        .nest_service("/public", ServeDir::new("./public"))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", args.port);
    info!("Listening on {}", addr);
    let cert = std::fs::read(format!("{}/cert.pem", args.cert))?;
    let key = std::fs::read(format!("{}/key.pem", args.cert))?;
    let config = RustlsConfig::from_pem(cert, key).await?;
    axum_server::bind_rustls(addr.parse()?, config)
        .serve(app.into_make_service())
        .await?;

    println!("Hello, world!");
    Ok(())
}
