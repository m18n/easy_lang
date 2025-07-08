use std::sync::Arc;
use bytes::Bytes;

use async_openai::{
    types::{
        ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs
    },
    Client,
};
use std::env;
use async_openai::config::OpenAIConfig;
use async_openai::types::{ChatCompletionResponseFormat, ChatCompletionResponseFormatType, CreateSpeechRequestArgs, SpeechModel, Voice};


use serde::de::DeserializeOwned;
use serde::Deserialize;
use sqlx::encode::IsNull::No;
use crate::base::get_nowtime_str;
use anyhow::{anyhow, Result};
use crate::controllers::object_of_controller::ResultGptTranslate;

pub struct GptModule{
    api:Client<OpenAIConfig>,
}
// Передаємо запитт
impl GptModule {
    pub async fn new(api_key:String)->Self{
        let config = OpenAIConfig::new().with_api_key(api_key);
        let client = Client::with_config(config);
        Self{
            api:client
        }
    }
    pub async fn text_to_audio(&self,text:String)-> Result<Bytes>{
        let request = CreateSpeechRequestArgs::default()
            .input(text)
            .voice(Voice::Nova)

            .model(SpeechModel::Tts1Hd)
            .build()?;

        let response = self.api.audio().speech(request).await?;
        Ok(response.bytes)
    }
    pub async fn send<T>(&self,request:String) -> Result<T>
        where
            T: DeserializeOwned,{
        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(512u16)
            .model("gpt-4o")
            .response_format(ChatCompletionResponseFormat{r#type:ChatCompletionResponseFormatType::JsonObject})
            .messages([
                ChatCompletionRequestSystemMessageArgs::default()
                    .content(request)
                    .build()?
                    .into(),
            ])
            .build()?;
        let response = self.api.chat().create(request).await?;

            let content=response.choices[0].message.content.clone().ok_or(anyhow!("gpt string parsing error"))?;
        // Deserialize the content string into type T
        let parsed_response:T = serde_json::from_str(&content)?;
        Ok(parsed_response)
    }
}