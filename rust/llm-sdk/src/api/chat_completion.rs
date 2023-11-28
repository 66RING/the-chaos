use serde::{Deserialize, Serialize};
use crate::IntoRequest;

#[derive(Serialize, Debug, Clone)]
pub struct ChatCompletionRequest {}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ChatCompletionResponse {}

impl IntoRequest for ChatCompletionRequest {
    fn into_request(self, client: reqwest::Client) -> crate::RequestBuilder {
        todo!()
    }
}
