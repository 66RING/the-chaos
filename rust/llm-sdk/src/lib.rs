mod api;

use std::{time::Duration};
use bytes::Bytes;
use anyhow::{Result, anyhow};
use schemars::{JsonSchema, schema_for};
use tracing::error;
use async_trait::async_trait;

use api::{ChatCompletionResponse, CreateImageRequest, CreateImageResponse, ChatCompletionRequest, CreateSpeechRequest, CreateTranscriptionRequest, CreateTranscriptionResponse, TranscriptionResponseFormat};
use reqwest::{Client, RequestBuilder, Response};

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
        let res = req.send_and_log().await?;
        Ok(res.json::<ChatCompletionResponse>().await?)
    }

    pub async fn create_image(&self, req: CreateImageRequest) -> Result<CreateImageResponse> {
        let req = self.prepare_request(req);
        let res = req.send_and_log().await?;
        Ok(res.json::<CreateImageResponse>().await?)
    }

    pub async fn create_speech(&self, req: CreateSpeechRequest) -> Result<Bytes> {
        let req = self.prepare_request(req);
        let res = req.send_and_log().await?;
        Ok(res.bytes().await?)
    }

    pub async fn create_transcription(&self, req: CreateTranscriptionRequest) -> Result<CreateTranscriptionResponse> {
        let is_json = req.response_format.is_some() && req.response_format.unwrap() == TranscriptionResponseFormat::Json;
        let req = self.prepare_request(req);
        let res = req.send_and_log().await?;
        let ret = if is_json {
            res.json::<CreateTranscriptionResponse>().await?
        } else {
            let text = res.text().await?;
            CreateTranscriptionResponse { text }
        };
        Ok(ret)
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

#[async_trait]
trait SendAndLog {
    async fn send_and_log(self) -> Result<Response>;
}

#[async_trait]
impl SendAndLog for RequestBuilder {
    async fn send_and_log(self) -> Result<Response> {
        let res = self.send().await?;
        let status = res.status();
        if status.is_client_error() || status.is_server_error() {
            let text = res.text().await?;
            error!("API error: {}", text);
            return Err(anyhow!("API error: {}", text));
        }
        Ok(res)
    }
}


pub trait ToSchema: JsonSchema {
    fn to_schema() -> serde_json::Value;
}

impl<T: JsonSchema> ToSchema for T {
    fn to_schema() -> serde_json::Value {
        serde_json::to_value(schema_for!(Self)).unwrap()
    }
}

#[cfg(test)]
#[ctor::ctor]
fn init() {
    tracing_subscriber::fmt::init();
}
