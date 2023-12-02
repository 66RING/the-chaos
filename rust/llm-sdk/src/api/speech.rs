use derive_builder::Builder;
use reqwest::{Client, RequestBuilder};
use serde::Serialize;

use crate::IntoRequest;

#[derive(Serialize, Debug, Clone, Default, Builder)]
#[builder(pattern = "mutable")]
pub struct CreateSpeechRequest {
    /// One of the available TTS models: tts-1 or tts-1-hd
    #[builder(default)]
    model: SpeechModel,
    /// The text to generate audio for. The maximum length is 4096 characters.
    #[builder(setter(into))]
    input: String,
    /// The voice to use when generating the audio. Supported voices are alloy, echo, fable, onyx, nova, and shimmer. Previews of the voices are available in the Text to speech guide.
    #[builder(default)]
    voice: SpeechVoice,
    /// The format to audio in. Supported formats are mp3, opus, aac, and flac.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<SpeechResponseFormat>,
    /// The speed of the generated audio. Select a value from 0.25 to 4.0. 1.0 is the default.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    speed: Option<f32>,
}

#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpeechResponseFormat {
    #[serde(rename = "mp3")]
    #[default]
    Mp3,
    #[serde(rename = "opus")]
    Opus,
    #[serde(rename = "aac")]
    Aac,
    #[serde(rename = "flac")]
    Flac,
}

#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpeechModel {
    #[serde(rename = "tts-1")]
    #[default]
    Tts1,
    #[serde(rename = "tts-1-hd")]
    Tts1Hd,
}

#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum SpeechVoice {
    Alloy,
    Echo,
    Fable,
    Onyx,
    #[default]
    Nova,
    Shimmer,
}

impl CreateSpeechRequest {
    pub fn new(input: impl Into<String>) -> Self {
        CreateSpeechRequestBuilder::default()
            .input(input)
            .build()
            .unwrap()
    }
}

impl IntoRequest for CreateSpeechRequest {
    fn into_request(self, base_url: &str, client: Client) -> RequestBuilder {
        let url = format!("{}/audio/speech", base_url);
        client.post(url).json(&self)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::SDK;
    use anyhow::Result;

    #[tokio::test]
    async fn speech_should_work() -> Result<()> {
        let req = CreateSpeechRequest::new("Hello, world!");
        let res = SDK.create_speech(req).await?;

        fs::write("/tmp/llm-sdk/speech.mp3", res)?;

        Ok(())
    }
}
