use derive_builder::Builder;
use reqwest::multipart::{Form, Part};
use reqwest::{Client, RequestBuilder};
use serde::Deserialize;
use strum::{Display, EnumString};

use crate::IntoRequest;

#[derive(Debug, Clone, Builder)]
#[builder(pattern = "mutable")]
pub struct CreateTranscriptionRequest {
    /// The audio file object (not file name) to transcribe, in one of these formats: flac, mp3, mp4, mpeg, mpga, m4a, ogg, wav, or webm.
    file: Vec<u8>,
    /// ID of the model to use. Only whisper-1 is currently available.
    #[builder(default)]
    model: TranscriptionModel,
    /// The language of the input audio. Supplying the input language in ISO-639-1 format will improve accuracy and latency.
    #[builder(default, setter(strip_option, into))]
    language: Option<String>,
    /// An optional text to guide the model's style or continue a previous audio segment. The prompt should match the audio language.
    #[builder(default, setter(strip_option, into))]
    prompt: Option<String>,
    /// The format of the transcript output, in one of these options: json, text, srt, verbose_json, or vtt.
    #[builder(default, setter(strip_option))]
    pub(crate) response_format: Option<TranscriptionResponseFormat>,
    /// The sampling temperature, between 0 and 1. Higher values like 0.8 will make the output more random, while lower values like 0.2 will make it more focused and deterministic. If set to 0, the model will use log probability to automatically increase the temperature until certain thresholds are hit.
    #[builder(default, setter(strip_option))]
    temperature: Option<f32>,
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default, EnumString, Display)]
#[strum(serialize_all = "snake_case")]
pub enum TranscriptionResponseFormat {
    #[default]
    Json,
    Text,
    Srt,
    VerboseJson,
    Vtt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, EnumString, Display)]
pub enum TranscriptionModel {
    #[default]
    #[strum(serialize = "whisper-1")]
    Whisper1,
}

impl CreateTranscriptionRequest {
    pub fn new(file: Vec<u8>) -> Self {
        CreateTranscriptionRequestBuilder::default()
            .file(file)
            .build()
            .unwrap()
    }

    pub fn into_form(self) -> Form {
        let part = Part::bytes(self.file)
            .file_name("file")
            .mime_str("audio/mp3")
            .unwrap();
        let form = Form::new()
            .part("file", part)
            .text("model", self.model.to_string());

        let form = if let Some(language) = self.language {
            form.text("language", language)
        } else {
            form
        };

        let form = if let Some(prompt) = self.prompt {
            form.text("prompt", prompt)
        } else {
            form
        };

        let form = if let Some(response_format) = self.response_format {
            form.text("response_format", response_format.to_string())
        } else {
            form
        };

        let form = if let Some(temperature) = self.temperature {
            form.text("temperature", temperature.to_string())
        } else {
            form
        };

        form
    }
}

impl IntoRequest for CreateTranscriptionRequest {
    fn into_request(self, client: Client) -> RequestBuilder {
        client
            .post("https://api.openai.com/v1/audio/transcriptions")
            .multipart(self.into_form())
        // TODO: review multipart
    }
}

#[derive(Deserialize, Debug, Clone, Builder)]
pub struct CreateTranscriptionResponse {
    /// The transcribed text.
    pub text: String,
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use anyhow::Result;

    #[tokio::test]
    async fn transcription_should_work() -> Result<()> {
        let sdk = crate::LlmSdk::new(std::env::var("OPENAI_API_KEY")?);
        let data = fs::read("/tmp/llm-sdk/speech.mp3")?;
        let req = CreateTranscriptionRequest::new(data);
        let res = sdk.create_transcription(req).await?;
        assert_eq!(res.text, "Hello, world.");
        Ok(())
    }

    #[tokio::test]
    async fn transcription_format_should_work() -> Result<()> {
        let sdk = crate::LlmSdk::new(std::env::var("OPENAI_API_KEY")?);
        let data = fs::read("/tmp/llm-sdk/speech.mp3")?;
        let req = CreateTranscriptionRequestBuilder::default()
            .file(data)
            .response_format(TranscriptionResponseFormat::Text)
            .build()?;

        let res = sdk.create_transcription(req).await?;
        assert_eq!(res.text, "Hello, world.\n");
        Ok(())
    }
}



