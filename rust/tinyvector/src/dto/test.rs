use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct TestPutRequest {
    #[serde(rename = "key1")]
    pub key11: String,
    #[serde(rename = "key2")]
    pub key22: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TestPostRequest {
    pub my_name: String,
    pub my_age: usize,
}
