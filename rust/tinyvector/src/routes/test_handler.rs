use axum::routing::{delete, get, post, put};
use axum::Router;
use tracing::{info, debug};

use crate::dto::test::{TestPutRequest, TestPostRequest};

pub struct DBHandler {}


impl DBHandler {
    pub fn handler() -> Router {
        Router::new()
            .route("/get", get(DBHandler::get))
            .route("/put", put(DBHandler::put_demo_json))
            .route("/post", post(DBHandler::post_demo_json))
            .route("/delete/:id", delete(DBHandler::delete_by_id))
    }

    /// Get handler
    pub async fn get() -> (axum::http::StatusCode, String) {
        info!("access /test/get");
        (axum::http::StatusCode::OK, "Everything is OK".to_string())
    }

    pub async fn put_demo_json(
        axum::extract::Json(data): axum::extract::Json<TestPutRequest>,
    ) -> String {
        info!("Put demo JSON data: {:?}", data);
        format!("Put demo JSON data: {:?}", data)
    }

    pub async fn post_demo_json(
        axum::extract::Json(data): axum::extract::Json<TestPostRequest>,
    ) -> String {
        info!("Post demo JSON data: {:?}", data);
        format!("Post demo JSON data: {:?}", data)
    }

    pub async fn delete_by_id(axum::extract::Path(id): axum::extract::Path<String>) -> String {
        info!("Get items with path id: {:?}", id);
        format!("Get items with path id: {:?}", id)
    }
}
