mod server;
mod routes;
mod dto;
mod database;
mod similarity;

mod test_data;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    server::start().await?;
    Ok(())
}
