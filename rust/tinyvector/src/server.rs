use crate::database::Database;
use crate::routes::db_handler::DbHandler;
use crate::routes::test_handler::TestHandler;
use anyhow::Result;
use tracing::info;

pub async fn start() -> Result<()> {
    let test_hander = TestHandler::handler();
    let db_hander = DbHandler::handler();

    let db = Database::load_from_file()?;

    // Build our application with a single route.
    let app = axum::Router::new()
        .nest("/test", test_hander)
        .nest("/db", db_hander);

    let app = app.layer(db.extension());

    // Run our application as a hyper server on http://localhost:3000.
    info!("listening at port 3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await?;
    Ok(())
}
