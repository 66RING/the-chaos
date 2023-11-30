use anyhow::Result;
use tracing::info;
use crate::routes::test_handler::DBHandler;

pub async fn start() -> Result<()> {
    let db_hander = DBHandler::handler();
    // Build our application with a single route.
    let app = axum::Router::new()
        .nest("/test", db_hander);

    // Run our application as a hyper server on http://localhost:3000.
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await?;
    Ok(())
}
