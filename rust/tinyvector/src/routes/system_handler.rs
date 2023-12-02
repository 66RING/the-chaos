use axum::{Router, Json};
use axum::routing::get;
use axum::Extension;
use crate::shutdown::Shutdown;

pub struct SystemHandler {}

impl SystemHandler {
    pub fn handler() -> Router {
        Router::new()
            .route("/shutdown", get(Self::shutdown))
    }

    async fn shutdown(Extension(shutdown): Extension<Shutdown>) -> Json<String> {
        shutdown.start_shutdown();
        Json("Shutting down...".to_string())
    }
}
