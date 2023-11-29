use serde::{Deserialize, Serialize};
use crate::IntoRequest;
use derive_builder::Builder;
use reqwest::{Client, RequestBuilder};

#[derive(Serialize, Debug, Clone, Builder)]
#[builder(pattern = "mutable")]
pub struct CreateImageRequest {
    /// A text description of the desired image(s). The maximum length is 4000 characters for dall-e-3.
    #[builder(setter(into))]
    prompt: String,
    /// The model to use for image generation. Only support dall-e-3
    #[builder(default)]
    model: ImageModel,
    /// The number of images to generate. Must be between 1 and 10. For dall-e-3, only n=1 is supported.
    /// set时自动剔除option
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    n: Option<usize>,
    /// The quality of the image that will be generated. hd creates images with finer details and greater consistency across the image. This param is only supported for dall-e-3.
    #[builder(default,setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    quality: Option<ImageQuality>,
    /// The format in which the generated images are returned. Must be one of url or b64_json.
    #[builder(default,setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<ImageResponseFormat>,
    /// The size of the generated images. Must be one of 256x256, 512x512, or 1024x1024 for dall-e-2. Must be one of 1024x1024, 1792x1024, or 1024x1792 for dall-e-3 models.
    #[builder(default,setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<ImageSize>,
    /// The style of the generated images. Must be one of vivid or natural. Vivid causes the model to lean towards generating hyper-real and dramatic images. Natural causes the model to produce more natural, less hyper-real looking images. This param is only supported for dall-e-3.
    #[builder(default,setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    style: Option<ImageStyle>,
    /// A unique identifier representing your end-user, which can help OpenAI to monitor and detect abuse.
    #[builder(default,setter(strip_option, into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    user: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateImageResponse {
    pub created: u64,
    pub data: Vec<ImageObject>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImageObject {
    /// The base64-encoded JSON of the generated image, if response_format is b64_json.
    pub b64_json: Option<String>,
    /// The URL of the generated image, if response_format is url (default).
    pub url: Option<String>,
    /// The prompt that was used to generate the image, if there was any revision to the prompt.
    pub revised_prompt: String,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ImageModel {
    #[serde(rename="dall-e-3")]
    #[default]
    DallE3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ImageQuality {
    #[default]
    Standard,
    Hd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ImageResponseFormat {
    #[default]
    Url,
    B64Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ImageSize {
    #[serde(rename = "1024x1024")]
    #[default]
    Large,
    #[serde(rename = "1792x1024")]
    LargeWide,
    #[serde(rename = "1024x1792")]
    LargeTall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ImageStyle {
    #[default]
    Vivid,
    Natural,
}

impl IntoRequest for CreateImageRequest {
    fn into_request(self, client: Client) -> RequestBuilder {
        client.post("https://api.openai.com/v1/images/generations")
            .json(&self)
    }
}

impl CreateImageRequest {
    pub fn new(prompt: impl Into<String>) -> Self {
        CreateImageRequestBuilder::default()
            .prompt(prompt)
            .build()
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::LlmSdk;

    use super::*;
    use anyhow::Result;
    use serde_json::json;

    #[test]
    fn create_image_request_should_serialize() -> Result<()> {
        // let req = CreateImageRequest::new("draw a cute cat");
        let req = CreateImageRequestBuilder::default()
            .prompt("draw a cute cat")
            .style(ImageStyle::Vivid)
            .quality(ImageQuality::Hd)
            .build()?;
        assert_eq!(
             serde_json::to_value(req)?,
             json!({
                "prompt": "draw a cute cat",
                "model": "dall-e-3",
                "style": "vivid",
                "quality": "hd",
             }),
        );
        Ok(())
    }

    #[tokio::test]
    async fn create_image_shoule_work() -> Result<()> {
        let sdk = LlmSdk::new(std::env::var("OPENAI_API_KEY")?);
        let req = CreateImageRequest::new("draw a cute cat");
        let res = sdk.create_image(req).await?;
        assert_eq!(res.data.len(), 1);
        let image = &res.data[0];
        assert!(image.url.is_some());
        assert!(image.b64_json.is_none());
        println!("image {:?}", image);
        fs::write("/tmp/llm-sdk/image.png", reqwest::get(image.url.as_ref().unwrap()).await?.bytes().await?)?;
        Ok(())
    }

}

