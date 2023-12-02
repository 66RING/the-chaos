use derive_builder::Builder;
use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};

use crate::IntoRequest;

#[derive(Serialize, Debug, Clone, Builder)]
#[builder(pattern = "mutable")]
pub struct CreateEmbeddingRequest {
    /// Input text to embed, encoded as a string or array of tokens. To embed multiple inputs in a single request, pass an array of strings or array of token arrays. The input must not exceed the max input tokens for the model (8192 tokens for text-embedding-ada-002), cannot be an empty string, and any array must be 2048 dimensions or less.
    input: EmbeddingInput,
    /// ID of the model to use. You can use the List models API to see all of your available models, or see our Model overview for descriptions of them.
    #[builder(default)]
    model: EmbeddingModel,
    /// The format to return the embeddings in. Can be either float or base64.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    encoding_format: Option<EmbeddingEncodingFormat>,
    /// A unique identifier representing your end-user, which can help OpenAI to monitor and detect abuse.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    user: Option<String>,
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub enum EmbeddingEncodingFormat {
    #[serde(rename = "float")]
    Float,
    #[serde(rename = "base64")]
    Base64,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub enum EmbeddingModel {
    #[serde(rename = "text-embedding-ada-002")]
    #[default]
    TextEmbeddingAda002,
}

#[derive(Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum EmbeddingInput {
    String(String),
    StringArray(Vec<String>),
}

#[derive(Deserialize, Debug, Clone)]
pub struct CreateEmbeddingResponse {
    pub object: String,
    pub data: Vec<EmbeddingData>,
    pub model: String,
    pub usage: EmbeddingUsage,
}

#[derive(Deserialize, Debug, Clone)]
pub struct EmbeddingUsage {
    pub prompt_tokens: usize,
    pub total_tokens: usize,
}

#[derive(Deserialize, Debug, Clone)]
pub struct EmbeddingData {
    /// The index of the embedding in the list of embeddings.
    pub index: usize,
    /// The embedding vector, which is a list of floats. The length of vector depends on the model as listed in the embedding guide.
    pub embedding: Vec<f32>,
    /// The object type, which is always "embedding".
    pub object: String,
}

impl CreateEmbeddingRequest {
    pub fn new(input: impl Into<EmbeddingInput>) -> Self {
        CreateEmbeddingRequestBuilder::default()
            .input(input.into())
            .build()
            .unwrap()
    }

    pub fn new_array(input: Vec<String>) -> Self {
        CreateEmbeddingRequestBuilder::default()
            .input(input.into())
            .build()
            .unwrap()
    }
}

impl From<String> for EmbeddingInput {
    fn from(s: String) -> Self {
        EmbeddingInput::String(s)
    }
}

impl From<&str> for EmbeddingInput {
    fn from(s: &str) -> Self {
        EmbeddingInput::String(s.to_string())
    }
}

impl From<Vec<String>> for EmbeddingInput {
    fn from(s: Vec<String>) -> Self {
        EmbeddingInput::StringArray(s)
    }
}

impl From<&[String]> for EmbeddingInput {
    fn from(s: &[String]) -> Self {
        EmbeddingInput::StringArray(s.to_vec())
    }
}

impl IntoRequest for CreateEmbeddingRequest {
    fn into_request(self, base_url: &str, client: Client) -> RequestBuilder {
        let url = format!("{}/embeddings", base_url);
        client.post(url).json(&self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SDK;
    use anyhow::Result;

    #[tokio::test]
    async fn embedding_should_work() -> Result<()> {
        let req = CreateEmbeddingRequest::new("hello");
        let res = SDK.create_embedding(req).await?;
        assert_eq!(res.data.len(), 1);
        assert_eq!(res.object, "list");
        // Response model id is different
        assert_eq!(res.model, "text-embedding-ada-002-v2");
        let data = &res.data[0];
        assert_eq!(data.embedding.len(), 1536);
        assert_eq!(data.object, "embedding");
        assert_eq!(data.index, 0);
        Ok(())
    }

    #[tokio::test]
    async fn array_embedding_should_work() -> Result<()> {
        let req =
            CreateEmbeddingRequest::new_array(vec!["hello world".into(), "宇宙的尽头".into()]);
        let res = SDK.create_embedding(req).await?;
        assert_eq!(res.data.len(), 2);
        assert_eq!(res.object, "list");
        // Response model id is different
        assert_eq!(res.model, "text-embedding-ada-002-v2");
        let data = &res.data[1];
        assert_eq!(data.embedding.len(), 1536);
        assert_eq!(data.object, "embedding");
        assert_eq!(data.index, 1);
        Ok(())
    }
}
