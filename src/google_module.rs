use std::collections::HashMap;
use std::sync::Arc;
use reqwest::Client;
use serde_json::{json, Value};
pub struct GoogleModule{
    api_key:String,
    client:Client
}
use base64::{decode, DecodeError};
use crate::base::get_nowtime_str;
use anyhow::{anyhow, Result};
fn decode_audio(audio_content_base64: &str) -> Result<Vec<u8>> {
   let res=decode(audio_content_base64)?;
    Ok(res)
}


pub fn get_language_code(language: String) -> Option<String> {
    let language_map: HashMap<&str, &str> = [
        ("Arabic", "ar-SA"),
        ("Dutch", "nl-NL"),
        ("English", "en-US"),
        ("French", "fr-FR"),
        ("German", "de-DE"),
        ("Greek", "el-GR"),
        ("Hebrew", "he-IL"),
        ("Italian", "it-IT"),
        ("Japanese", "ja-JP"),
        ("Korean", "ko-KR"),
        ("Polish", "pl-PL"),
        ("Portuguese", "pt-PT"),
        ("Russian", "ru-RU"),
        ("Spanish", "es-ES"),
        ("Swedish", "sv-SE"),
        ("Thai", "th-TH"),
        ("Turkish", "tr-TR"),
        ("Ukrainian", "uk-UA"),
        ("Vietnamese", "vi-VN"),
    ].iter().cloned().collect();
    let res=language_map.get(language.as_str()).copied();
    match res {
        None => {
            None
        }
        Some(str) => {
            Some(String::from(str))
        }
    }

}
impl GoogleModule{
    pub fn new(api_key:String)->GoogleModule{
        Self{
            api_key,
            client:Client::new()
        }
    }
    pub async fn text_to_speach(&self,text:String,name_lang:String)->Result<Vec<u8>> {
        let url = "https://texttospeech.googleapis.com/v1/text:synthesize";
        //sv-SE
        let body = json!({
        "input": {
            "text": text
        },
        "voice": {
            "languageCode": get_language_code(name_lang).unwrap(),
            "ssmlGender": "FEMALE"
        },
        "audioConfig": {
            "audioEncoding": "MP3"
        }
    });

        let response = self.client.post(url)
            .query(&[("key", self.api_key.as_str())])
            .json(&body)
            .send()
            .await?;

        if response.status().is_success() {
            let audio_content = response.text().await?;
            let parsed_json: Value = serde_json::from_str(audio_content.as_str())?;
            let audio_content = parsed_json["audioContent"].as_str().unwrap_or("");
            decode_audio(audio_content)
        } else {
            Err(anyhow!("respons to google is not successful"))
        }
    }
}