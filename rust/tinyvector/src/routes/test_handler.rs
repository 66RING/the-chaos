use axum::routing::{delete, get, post, put};
use axum::Router;
use tracing::{info};

use crate::dto::{TestPutRequest, TestPostRequest};

pub struct TestHandler {}


impl TestHandler {
    pub fn handler() -> Router {
        Router::new()
            .route("/get", get(TestHandler::get))
            .route("/put", put(TestHandler::put_demo_json))
            .route("/post", post(TestHandler::post_demo_json))
            .route("/delete/:id", delete(TestHandler::delete_by_id))
    }

    /// Get handler
    async fn get() -> (axum::http::StatusCode, String) {
        info!("access /test/get");
        (axum::http::StatusCode::OK, "Everything is OK".to_string())
    }

    async fn put_demo_json(
        axum::extract::Json(data): axum::extract::Json<TestPutRequest>,
    ) -> String {
        info!("Put demo JSON data: {:?}", data);
        format!("Put demo JSON data: {:?}", data)
    }

    async fn post_demo_json(
        axum::extract::Json(data): axum::extract::Json<TestPostRequest>,
    ) -> String {
        info!("Post demo JSON data: {:?}", data);
        format!("Post demo JSON data: {:?}", data)
    }

    async fn delete_by_id(axum::extract::Path(id): axum::extract::Path<String>) -> String {
        info!("Get items with path id: {:?}", id);
        format!("Get items with path id: {:?}", id)
    }
}
