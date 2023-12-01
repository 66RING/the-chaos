use std::collections::HashMap;

use crate::database::{DBError, Database, DbExtension, EmbeddingRecord, Table};
use crate::dto::*;
use crate::routes::helper::*;
use axum::extract::{Json, Path, Query};
use axum::http::StatusCode;
use axum::routing::{delete, get, post};
use axum::{Extension, Router};
use tracing::info;

pub struct DbHandler {}

impl DbHandler {
    pub fn handler() -> Router {
        Router::new()
            .route("/create_table", post(Self::create_table))
            .route("/drop_table/:table_name", delete(Self::drop_table))
            .route("/insert_record", post(Self::insert_record))
            .route(
                "/delete_record/:table_name/:id",
                delete(Self::delete_record),
            )
            .route("/query_record", post(Self::query_record))
            .route("/get_entire_db", get(Self::get_entire_db))
    }

    async fn create_table(
        Extension(db): DbExtension,
        Json(data): Json<CreateTableRequest>,
    ) -> (StatusCode, Json<BaseHttpResponse<Result<(), DBError>>>) {
        info!("Create table: {:?}", data);
        let res = {
            let mut db = db.write().await;
            db.create_table(data.table_name, data.dimension, data.distance)
        };

        if res.is_err() {
            return (StatusCode::BAD_REQUEST, Json(generate_base_response(res, false, 0)));
        }

        (StatusCode::OK, Json(generate_base_response(res, true, 0)))
    }

    async fn drop_table(
        Extension(db): DbExtension,
        Path(data): Path<DropTableRequest>,
    ) -> (StatusCode, Json<BaseHttpResponse<Result<(), DBError>>>) {
        info!("Drop table: {:?}", data);

        let res = {
            let mut db = db.write().await;
            db.drop_table(data.table_name)
        };

        if res.is_err() {
            return (StatusCode::OK, Json(generate_base_response(res, false, 0)));
        }

        (StatusCode::OK, Json(generate_base_response(res, true, 0)))
    }
    async fn insert_record(
        Extension(db): DbExtension,
        Json(data): Json<InsertRecordRequest>,
    ) -> (StatusCode, Json<BaseHttpResponse<Result<(), DBError>>>) {
        info!("Inserd record table name: {:?}, record id: {:?}", data.table_name, data.record.id);
        let res = {
            let mut db = db.write().await;
            db.insert_record(data.table_name, data.record)
        };

        if res.is_err() {
            return (StatusCode::OK, Json(generate_base_response(res, false, 0)));
        }

        (StatusCode::OK, Json(generate_base_response(res, true, 0)))
    }
    async fn delete_record(
        Extension(db): DbExtension,
        Path(data): Path<DeleteRecordRequest>,
    ) -> (StatusCode, Json<BaseHttpResponse<Result<(), DBError>>>) {
        info!("Delete record: {:?}", data);
        let res = {
            let mut db = db.write().await;
            db.delete_record(data.table_name, data.id)
        };

        if res.is_err() {
            return (StatusCode::OK, Json(generate_base_response(res, false, 0)));
        }

        (StatusCode::OK, Json(generate_base_response(res, true, 0)))
    }
    async fn query_record(
        Extension(db): DbExtension,
        Json(data): Json<QueryRecordRequest>,
    ) -> (StatusCode, Json<BaseHttpResponse<Result<Vec<EmbeddingRecord>, DBError>>>) {
        info!("Query record: table name: {:?}, top k: {:?}", data.table_name, data.top_k);
        let res = {
            let db = db.read().await;
            db.query_record(data.table_name, &data.query_embedding, data.top_k)
        };

        if res.is_err() {
            return (StatusCode::OK, Json(generate_base_response(res, false, 0)));
        }

        (StatusCode::OK, Json(generate_base_response(res, true, 0)))
    }
    async fn get_entire_db(
        Extension(db): DbExtension,
    ) -> (StatusCode, Json<BaseHttpResponse<Result<HashMap<String, Table>, DBError>>>) {
        let res = {
            let db = db.read().await;
            db.get_entire_db()
        };

        if res.is_err() {
            return (StatusCode::OK, Json(generate_base_response(res, false, 0)));
        }

        (StatusCode::OK, Json(generate_base_response(res, true, 0)))
    }
}
