use derive_builder::Builder;
use reqwest::multipart::{Form, Part};
use reqwest_middleware::{ClientWithMiddleware, RequestBuilder};
use serde::Deserialize;
use strum::{Display, EnumString};

use crate::IntoRequest;

#[derive(Debug, Clone, Builder)]
#[builder(pattern = "mutable")]
pub struct CreateWhisperRequest {
    /// The audio file object (not file name) to transcribe, in one of these formats: flac, mp3, mp4, mpeg, mpga, m4a, ogg, wav, or webm.
    file: Vec<u8>,
    /// ID of the model to use. Only whisper-1 is currently available.
    #[builder(default)]
    model: WhisperModel,
    /// The language of the input audio. Supplying the input language in ISO-639-1 format will improve accuracy and latency.
    #[builder(default, setter(strip_option, into))]
    language: Option<String>,
    /// An optional text to guide the model's style or continue a previous audio segment. The prompt should match the audio language.
    #[builder(default, setter(strip_option, into))]
    prompt: Option<String>,
    /// The format of the transcript output, in one of these options: json, text, srt, verbose_json, or vtt.
    #[builder(default, setter(strip_option))]
    pub(crate) response_format: Option<WhisperResponseFormat>,
    /// The sampling temperature, between 0 and 1. Higher values like 0.8 will make the output more random, while lower values like 0.2 will make it more focused and deterministic. If set to 0, the model will use log probability to automatically increase the temperature until certain thresholds are hit.
    #[builder(default, setter(strip_option))]
    temperature: Option<f32>,

    /// Request type: transcription or translation.
    request_type: WhisperRequestType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, EnumString, Display)]
pub enum WhisperRequestType {
    #[default]
    Transcription,
    Translation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, EnumString, Display)]
#[strum(serialize_all = "snake_case")]
pub enum WhisperResponseFormat {
    #[default]
    Json,
    Text,
    Srt,
    VerboseJson,
    Vtt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, EnumString, Display)]
pub enum WhisperModel {
    #[default]
    #[strum(serialize = "whisper-1")]
    Whisper1,
}

impl CreateWhisperRequest {
    pub fn transcription(file: Vec<u8>) -> Self {
        CreateWhisperRequestBuilder::default()
            .file(file)
            .request_type(WhisperRequestType::Transcription)
            .build()
            .unwrap()
    }

    pub fn translation(file: Vec<u8>) -> Self {
        CreateWhisperRequestBuilder::default()
            .file(file)
            .request_type(WhisperRequestType::Translation)
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

        // Translation doesn't have any language.
        let form = match (self.request_type, self.language) {
            (WhisperRequestType::Transcription, Some(language)) => form.text("language", language),
            _ => form,
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

        if let Some(temperature) = self.temperature {
            form.text("temperature", temperature.to_string())
        } else {
            form
        }
    }
}

impl IntoRequest for CreateWhisperRequest {
    fn into_request(self, base_url: &str, client: ClientWithMiddleware) -> RequestBuilder {
        let url = match self.request_type {
            WhisperRequestType::Transcription => format!("{}/audio/transcriptions", base_url),
            WhisperRequestType::Translation => format!("{}/audio/translations", base_url),
        };
        client.post(url).multipart(self.into_form())
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
    use crate::SDK;
    use anyhow::Result;

    #[tokio::test]
    async fn transcription_should_work() -> Result<()> {
        let data = fs::read("/tmp/llm-sdk/speech.mp3")?;
        let req = CreateWhisperRequest::transcription(data);
        let res = SDK.create_whisper(req).await?;
        assert_eq!(res.text, "{\n  \"text\": \"Hello, world.\"\n}");
        Ok(())
    }

    #[tokio::test]
    async fn transcription_format_should_work() -> Result<()> {
        let data = fs::read("/tmp/llm-sdk/speech.mp3")?;
        let req = CreateWhisperRequestBuilder::default()
            .file(data)
            .response_format(WhisperResponseFormat::Text)
            .request_type(WhisperRequestType::Transcription)
            .build()?;

        let res = SDK.create_whisper(req).await?;
        assert_eq!(res.text, "Hello, world.\n");
        Ok(())
    }

    #[tokio::test]
    async fn translation_should_work() -> Result<()> {
        let data = fs::read("/tmp/llm-sdk/chinese.mp3")?;
        let req = CreateWhisperRequestBuilder::default()
            .file(data)
            .request_type(WhisperRequestType::Translation)
            .build()?;

        let res = SDK.create_whisper(req).await?;
        assert_eq!(res.text, "{\n  \"text\": \"The red scarf hangs on the chest, the motherland is always in my heart.\"\n}");
        Ok(())
    }
}
