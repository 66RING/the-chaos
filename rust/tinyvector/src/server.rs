use crate::database::Database;
use crate::routes::db_handler::DbHandler;
use crate::routes::test_handler::TestHandler;
use crate::routes::SystemHandler;
use crate::shutdown::Shutdown;
use anyhow::Result;
use axum::Server;
use std::net::SocketAddr;
use tracing::info;

pub async fn start() -> Result<()> {
    let test_hander = TestHandler::handler();
    let db_hander = DbHandler::handler();
    let system_handler = SystemHandler::handler();

    let db = Database::load_from_file()?;
    let shutdown = Shutdown::new()?;

    // Build our application with a single route.
    let app = axum::Router::new()
        .nest("/test", test_hander)
        .nest("/db", db_hander)
        .nest("/system", system_handler);

    let app = app.layer(db.extension()).layer(shutdown.extension());

    // Run our application as a hyper server on http://localhost:3000.
    info!("listening at port 3000");
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown.wait())
        .await?;
    // axum::serve(listener, app)
    //     .await?;
    Ok(())
}
