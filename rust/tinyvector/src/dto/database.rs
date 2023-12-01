use serde::{Deserialize, Serialize};
use crate::{similarity::Distance, database::EmbeddingRecord};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct CreateTableRequest {
    pub table_name: String,
    pub dimension: usize,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct DropTableRequest {
    pub table_name: String,
}


#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct InsertRecordRequest {
    pub table_name: String,
    pub record: EmbeddingRecord,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct DeleteRecordRequest {
    pub table_name: String,
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct QueryRecordRequest {
    pub table_name: String,
    pub query_embedding: Vec<f32>,
    pub top_k: usize,
    pub distance: Distance,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct QueryTableRequest {
    pub table_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct GetEntireDbRequest {
}

