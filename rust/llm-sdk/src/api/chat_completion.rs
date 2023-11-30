use crate::IntoRequest;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Clone, Builder)]
pub struct ChatCompletionRequest {
    /// A list of messages comprising the conversation so far.
    #[builder(setter(into))]
    messages: Vec<ChatCompletionMessage>,
    /// ID of the model to use. See the model endpoint compatibility table for details on which models work with the Chat API.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<ChatCompletionModel>,
    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on their existing frequency in the text so far, decreasing the model's likelihood to repeat the same line verbatim.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    frequency_penalty: Option<f32>,

    // Modify the likelihood of specified tokens appearing in the completion.
    // Accepts a JSON object that maps tokens (specified by their token ID in the tokenizer) to an associated bias value from -100 to 100. Mathematically, the bias is added to the logits generated by the model prior to sampling. The exact effect will vary per model, but values between -1 and 1 should decrease or increase likelihood of selection; values like -100 or 100 should result in a ban or exclusive selection of the relevant token.
    // #[builder(setter(strip_option))]
    // #[serde(skip_serializing_if = "Option::is_none")]
    // logit_bias: Option<f32>,
    /// The maximum number of tokens to generate in the chat completion.
    /// The total length of input tokens and generated tokens is limited by the model's context length.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<usize>,
    /// How many chat completion choices to generate for each input message. Note that you will be charged based on the number of generated tokens across all of the choices. Keep n as 1 to minimize costs.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    n: Option<usize>,
    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on whether they appear in the text so far, increasing the model's likelihood to talk about new topics.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    presence_penalty: Option<f32>,
    /// An object specifying the format that the model must output.
    /// Setting to { "type": "json_object" } enables JSON mode, which guarantees the message the model generates is valid JSON.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<ChatCompletionResponseFormatObject>,
    /// This feature is in Beta. If specified, our system will make a best effort to sample deterministically, such that repeated requests with the same seed and parameters should return the same result. Determinism is not guaranteed, and you should refer to the system_fingerprint response parameter to monitor changes in the backend.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<usize>,
    /// Up to 4 sequences where the API will stop generating further tokens.
    // TODO: make this as enum
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<String>,
    /// If set, partial message deltas will be sent, like in ChatGPT. Tokens will be sent as data-only server-sent events as they become available, with the stream terminated by a data: [DONE] message.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,

    /// What sampling temperature to use, between 0 and 2. Higher values like 0.8 will make the output more random, while lower values like 0.2 will make it more focused and deterministic.
    /// We generally recommend altering this or top_p but not both.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    /// An alternative to sampling with temperature, called nucleus sampling, where the model considers the results of the tokens with top_p probability mass. So 0.1 means only the tokens comprising the top 10% probability mass are considered.
    /// We generally recommend altering this or temperature but not both.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    /// A list of tools the model may call. Currently, only functions are supported as a tool. Use this to provide a list of functions the model may generate JSON inputs for.
    #[builder(default, setter(into))]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tool: Vec<Tool>,
    /// Controls which (if any) function is called by the model. none means the model will not call a function and instead generates a message. auto means the model can pick between generating a message or calling a function. Specifying a particular function via {"type: "function", "function": {"name": "my_function"}} forces the model to call that function.
    /// none is the default when no functions are present. auto is the default if functions are present.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<ToolChoice>,
    /// A unique identifier representing your end-user, which can help OpenAI to monitor and detect abuse.
    #[builder(default, setter(strip_option, into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    user: Option<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ToolChoice {
    #[default]
    None,
    Auto,

    // TODO: we need something like this: #[serde(tag = "type", content = "function")]
    Function {
        name: String,
    },
}

#[derive(Clone, Debug, Serialize)]
pub struct Tool {
    /// The type of the tool. Currently, only function is supported.
    r#type: ToolType,
    function: FunctionInfo,
}

#[derive(Clone, Debug, Serialize)]
pub struct FunctionInfo {
    /// A description of what the function does, used by the model to choose when and how to call the function.
    description: String,
    /// The name of the function to be called. Must be a-z, A-Z, 0-9, or contain underscores and dashes, with a maximum length of 64.
    name: Option<String>,
    /// The parameters the functions accepts, described as a JSON Schema object. See the guide for examples, and the JSON Schema reference for documentation about the format.
    /// To describe a function that accepts no parameters, provide the value {"type": "object", "properties": {}}.
    parameters: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq, Default)]
pub struct ChatCompletionResponseFormatObject {
    r#type: ChatCompletionResponseFormat,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq, Default)]
pub enum ChatCompletionResponseFormat {
    Text,
    #[default]
    Json,
}

#[derive(Clone, Debug, Serialize)]
pub enum ChatCompletionModel {
    #[serde(rename = "gpt-3.5-1106-turbo-1106")]
    Gpt3Turbo,
    #[serde(rename = "gpt-4-1106-preview")]
    Gpt4Turbo,
    #[serde(rename = "gpt-4-1106-vision-preview")]
    Gpt4TurboVision,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case", tag = "role")]
pub enum ChatCompletionMessage {
    /// A message from system.
    System(SystemMessage),
    /// A message from user.
    User(UserMessage),
    /// A message from the assistant.
    Assistant(AssistantMessage),
    /// A message from tool.
    Tool(ToolMessage),
}

#[derive(Clone, Debug, Serialize)]
pub struct SystemMessage {
    /// The contents of the system message.
    content: String,
    /// An optional name for the participant. Provides the model information to differentiate between participants of the same role.
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct UserMessage {
    /// The contents of the user message.
    content: String,
    /// An optional name for the participant. Provides the model information to differentiate between participants of the same role.
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AssistantMessage {
    /// The contents of the assistant message.
    content: String,
    /// An optional name for the participant. Provides the model information to differentiate between participants of the same role.
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    /// The tool calls generated by the model, such as function calls.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tool_calls: Vec<ToolCalls>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ToolCalls {
    /// The ID of the tool call.
    id: String,
    /// The type of the tool. Currently, only function is supported.
    r#type: ToolType,
    /// The function that the model called.
    function: FunctionCall,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ToolType {
    Function,
}

#[derive(Clone, Debug, Serialize)]
pub struct FunctionCall {
    /// The name of the function to call.
    name: String,
    /// The arguments to call the function with, as generated by the model in JSON format. Note that the model does not always generate valid JSON, and may hallucinate parameters not defined by your function schema. Validate the arguments in your code before calling your function.
    arguments: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct ToolMessage {
    /// The contents of the tool message.
    content: String,
    /// An optional name for the participant. Provides the model information to differentiate between participants of the same role.
    name: Option<String>,
    /// Tool call that this message is responding to.
    tool_call_id: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ChatCompletionResponse {}

impl IntoRequest for ChatCompletionRequest {
    fn into_request(self, client: reqwest::Client) -> crate::RequestBuilder {
        todo!()
    }
}

impl ChatCompletionMessage {
    pub fn new_system(content: impl Into<String>, name: &str) -> ChatCompletionMessage {
        ChatCompletionMessage::System(SystemMessage {
            content: content.into(),
            name: Self::get_name(name),
        })
    }
    pub fn new_user(content: impl Into<String>, name: &str) -> ChatCompletionMessage {
        ChatCompletionMessage::User(UserMessage {
            content: content.into(),
            name: Self::get_name(name),
        })
    }

    fn get_name(name: &str) -> Option<String> {
        if name.is_empty() {
            None
        } else {
            Some(name.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn tool_choice_serialize_function_should_work() {
        let req = ChatCompletionRequestBuilder::default()
            .tool_choice(ToolChoice::Function {
                name: "my_function".to_string(),
            })
            .messages(vec![])
            .build()
            .unwrap();

        let json = serde_json::to_value(req).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
              "tool_choice": {
                "type": "function",
                "function": {
                  "name": "my_function"
                }
              },
              "messages": []
            })
        )
    }

    #[test]
    fn tool_choice_serialize_auto_should_work() {
        let req = ChatCompletionRequestBuilder::default()
            .tool_choice(ToolChoice::Auto)
            .messages(vec![])
            .build()
            .unwrap();

        let json = serde_json::to_value(req).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
              "tool_choice": "auto",
              "messages": []
            })
        )
    }

    #[test]
    fn tool_choice_serialize_should_work() {
        let messages = vec![
            ChatCompletionMessage::new_system("I can answer any question you ask me.", ""),
            ChatCompletionMessage::new_user("What is human life expectancy in the world?", "user1"),
        ];
        let req = ChatCompletionRequestBuilder::default()
            .tool_choice(ToolChoice::Auto)
            .messages(messages)
            .build()
            .unwrap();

        let json = serde_json::to_value(req).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
              "tool_choice": "auto",
              "messages": [
                {
                  "content": "I can answer any question you ask me.",
                  "role": "system",
                },
                {
                  "content": "What is human life expectancy in the world?",
                  "role": "user",
                  "name": "user1"
              }]
            })
        )
    }
}
