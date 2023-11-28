mod api;

use std::time::Duration;
// TODO: review
use anyhow::Result;

use api::{ChatCompletionResponse, CreateImageRequest, CreateImageResponse, ChatCompletionRequest};
use reqwest::{Client, RequestBuilder};

const TIMEOUT: u64 = 30;

#[derive(Debug)]
pub struct LlmSdk {
    // pub in this crate only
    pub(crate) token: String,
    pub(crate) client: Client,
}

pub trait IntoRequest {
    fn into_request(self, client: Client) -> RequestBuilder;
}

impl LlmSdk {
    pub fn new(token: String) -> Self {
        Self {
            token,
            client: Client::new(),
        }
    }

    pub async fn chat_completion(&self, req: ChatCompletionRequest) -> Result<ChatCompletionResponse> {
        let req = self.prepare_request(req);
        let res = req.send().await?;
        Ok(res.json::<ChatCompletionResponse>().await?)
    }

    pub async fn create_image(&self, req: CreateImageRequest) -> Result<CreateImageResponse> {
        let req = self.prepare_request(req);
        let res = req.send().await?;
        Ok(res.json::<CreateImageResponse>().await?)
    }

    fn prepare_request(&self, req: impl IntoRequest) -> RequestBuilder {
        let req = req.into_request(self.client.clone());
        let req = if self.token.is_empty() {
            req
        } else {
            req.bearer_auth(&self.token)
        };
        req.timeout(Duration::from_secs(TIMEOUT))
    }
}
