use std::env;
use actix_web::{get, HttpMessage, HttpRequest, HttpResponse, post, Responder, web};
use actix_web::http::header;
use async_std::path::PathBuf;
use std::fs;
use crate::base::{get_nowtime_str};
use crate::controllers::object_of_controller::{CurrentLanguage, RequestResult, ResultGptTranscript, ResultGptTranslate, ResultTranslate, Sentences, Translate, TranslateGpt};
use crate::cookie::{create_cookie_auth, create_cookie_auth_clear};
use crate::generate_anki::generate_anki;
use crate::gpt_module::GptModule;
use crate::jwt::{Claims};
use crate::models::{Dictionary_Sentence, DataBase, SentenceId, Translated, TranslatedId};
use crate::StateDb;
use crate::translate_module::DeeplModule;
use anyhow::{anyhow, Result as AnyhowResult};
use crate::globals::Result;
use futures_util::SinkExt;

// url controller: /api/userspace/***
#[get("/test")]
pub async fn m_test()->Result<HttpResponse>{
    Ok(HttpResponse::Ok().content_type("text/html").body("Hello"))
}
#[post("/set-current-dictionary")]
pub async fn set_current_dictionary(req:HttpRequest, current_lang:web::Json<CurrentLanguage>, state: web::Data<StateDb>) ->Result<HttpResponse>{
    let claims=req.extensions().get::<Claims>().ok_or(anyhow!("dont find cookie"))?.clone();
        let mut new_index:usize=0;
        let result = claims.user_dictionaries.iter().enumerate().find(|(_, &ref x)| x.language_name==current_lang.current_lang);
        match result {
            Some((index, element)) => {
                new_index=index;
            },
            None => {

            }
        }
        let my_claims=Claims{

            current_lang_index:new_index,
            ..claims.clone()
        };
        let cookie=create_cookie_auth(my_claims.clone());
        let mut respon = HttpResponse::Ok().cookie(cookie).json(RequestResult { status: true });
        Ok(respon)


}

#[get("/logout")]
pub async fn logout(state: web::Data<StateDb>) ->Result<HttpResponse>{
    let cookie =create_cookie_auth_clear();
    let respon=HttpResponse::Found()
        .insert_header((header::LOCATION, "/login"))
        .cookie(cookie)
        .finish();
    Ok(respon)
}
#[post("/translate/deepl")]
pub async fn deepl_translate(req:HttpRequest, translate_info:web::Json<Translate>, state: web::Data<StateDb>) ->Result<HttpResponse>{
    let text_= state.deepl_api.translate(translate_info.from_lang.clone(),translate_info.into_lang.clone(),translate_info.text.clone()).await?;
    Ok(HttpResponse::Ok().json(ResultTranslate{text:text_}))
}
#[post("/translate/gpt/full/conversational")]
pub async fn gpt_translate_full_conversational(req:HttpRequest, translate_info:web::Json<TranslateGpt>, state: web::Data<StateDb>) ->Result<HttpResponse>{
    let query=format!("Будь вчителем {} мови.
Відповідайте лише у форматі JSON із такими даними:
{{
      \"sentence:\"\",
      \"explanation\":\"\"
}}
Як сказати моє речення {} мовою, розмовною {} мовою, щоб передати той самий зміст, та сенс. Моє речення написане {} мовою: \"{}\".
Я також наведу вам значення речення, яке я хотів передати: \"{}\".
У відповідь:
Поле \"sentence\" має містити твоє речення яке ти створив. Поле \"explanation\" повинно містити коротке пояснення вашого стовреного речення, твоє поясення має бути написано {} мовою.",
   translate_info.into_lang.clone(),translate_info.into_lang.clone(),translate_info.into_lang.clone(),translate_info.from_lang.clone(),translate_info.text.clone(),
    translate_info.text_explain.clone(),translate_info.from_lang.clone());
    let translate:ResultGptTranslate=state.gpt_api.send(query).await?;
    // let res_err=ResultGptTranslate{sentence:"Error, please try again".to_string(),explanation:"Error, please try again".to_string()};
    // Ok(HttpResponse::Ok().json(res_err))
    Ok(HttpResponse::Ok().json(translate))
}
#[post("/translate/gpt/full/formal")]
pub async fn gpt_translate_full_formal(req:HttpRequest, translate_info:web::Json<TranslateGpt>, state: web::Data<StateDb>) ->Result<HttpResponse>{
    let query=format!("Будь вчителем {} мови.
Відповідайте лише у форматі JSON із такими даними:
{{
      \"sentence:\"\",
      \"explanation\":\"\"
}}
Як сказати моє речення {} мовою, формальною {} мовою, щоб передати той самий зміст, та сенс. Моє речення написане {} мовою: \"{}\".
Я також наведу вам значення речення, яке я хотів передати: \"{}\".
У відповідь:
Поле \"sentence\" має містити твоє речення яке ти створив. Поле \"explanation\" повинно містити коротке пояснення вашого стовреного речення, твоє поясення має бути написано {} мовою.",
                      translate_info.into_lang.clone(),translate_info.into_lang.clone(),translate_info.into_lang.clone(),translate_info.from_lang.clone(),translate_info.text.clone(),
                      translate_info.text_explain.clone(),translate_info.from_lang.clone());
    let translate=state.gpt_api.send(query).await?;
    Ok(HttpResponse::Ok().json(translate))
    // let res_err=ResultGptTranslate{sentence:"Error, please try again".to_string(),explanation:"Error, please try again".to_string()};
    // Ok(HttpResponse::Ok().json(res_err))

}
#[post("/translate/gpt/short/conversational")]
pub async fn gpt_translate_short_conversational(req:HttpRequest, translate_info:web::Json<TranslateGpt>, state: web::Data<StateDb>) ->Result<HttpResponse>{
    let query=format!("Будь вчителем {} мови.
Відповідайте лише у форматі JSON із такими даними:
{{
      \"sentence:\"\",
      \"explanation\":\"\"
}}
Як сказати моє речення {} мовою, розмовною {} мовою, щоб передати той самий зміст, та сенс. Моє речення написане {} мовою: \"{}\".
Я також наведу вам значення речення, яке я хотів передати: \"{}\".
По можливості скороти речення, але щоб зміст та сенс не втратився.
У відповідь:
Поле \"sentence\" має містити твоє речення яке ти створив. Поле \"explanation\" повинно містити коротке пояснення вашого стовреного речення, твоє поясення має бути написано {} мовою.",
                      translate_info.into_lang.clone(),translate_info.into_lang.clone(),translate_info.into_lang.clone(),translate_info.from_lang.clone(),translate_info.text.clone(),
                      translate_info.text_explain.clone(),translate_info.from_lang.clone());
    let translate:ResultGptTranslate=state.gpt_api.send(query).await?;
    // let res_err=ResultGptTranslate{sentence:"Error, please try again".to_string(),explanation:"Error, please try again".to_string()};
    // Ok(HttpResponse::Ok().json(res_err))
    Ok(HttpResponse::Ok().json(translate))
}
#[post("/translate/gpt/short/formal")]
pub async fn gpt_translate_short_formal(req:HttpRequest, translate_info:web::Json<TranslateGpt>, state: web::Data<StateDb>) ->Result<HttpResponse>{
    let query=format!("Будь вчителем {} мови.
Відповідайте лише у форматі JSON із такими даними:
{{
      \"sentence:\"\",
      \"explanation\":\"\"
}}
Як сказати моє речення {} мовою, формальною {} мовою, щоб передати той самий зміст, та сенс. Моє речення написане {} мовою: \"{}\".
Я також наведу вам значення речення, яке я хотів передати: \"{}\".
По можливості скороти речення, але щоб зміст та сенс не втратився.
У відповідь:
Поле \"sentence\" має містити твоє речення яке ти створив. Поле \"explanation\" повинно містити коротке пояснення вашого стовреного речення, твоє поясення має бути написано {} мовою.",
                      translate_info.into_lang.clone(),translate_info.into_lang.clone(),translate_info.into_lang.clone(),translate_info.from_lang.clone(),translate_info.text.clone(),
                      translate_info.text_explain.clone(),translate_info.from_lang.clone());
    let translate:ResultGptTranslate=state.gpt_api.send(query).await?;
    // let res_err=ResultGptTranslate{sentence:"Error, please try again".to_string(),explanation:"Error, please try again".to_string()};
    // Ok(HttpResponse::Ok().json(res_err))
    Ok(HttpResponse::Ok().json(translate))
}
#[post("/translate/save")]
pub async fn save_translation(req:HttpRequest, translate_info:web::Json<Translated>, state: web::Data<StateDb>) ->Result<HttpResponse>{
    let claims=req.extensions().get::<Claims>().ok_or(anyhow!("dont find cookie"))?.clone();
    state.database.save_translate(translate_info.into_inner(), claims.user_id).await?;
    Ok(HttpResponse::Ok().json(RequestResult{status:true}))
}
#[post("/translate/history/delete")]
pub async fn delete_translation(req:HttpRequest, translate_info:web::Json<TranslatedId>, state: web::Data<StateDb>) ->Result<HttpResponse>{
    let claims=req.extensions().get::<Claims>().ok_or(anyhow!("dont find cookie"))?.clone();
    state.database.delete_translated(translate_info.into_inner(), claims.user_id).await?;
    Ok(HttpResponse::Ok().json(RequestResult{status:true}))
}

#[post("/dictionary/add-sentence")]
pub async fn add_dictionary_sentence(req:HttpRequest, sentences_info:web::Json<Sentences>, state: web::Data<StateDb>) ->Result<HttpResponse>{
    let claims=req.extensions().get::<Claims>().ok_or(anyhow!("dont find cookie"))?.clone();
    let query=format!("Напиши дві транскрипції для цього {} речення \"{}\". \
        Одна траксрипція це звичайна IPA а друга це адаптована під укараїнську мову.\
        Відповідь надай в JSON. У форматі об'єкту:\
        {{
            \"ipa\":\"\",
            \"ipa_ukr\":\"\",
        }}
        ",claims.user_dictionaries[claims.current_lang_index].language_name,sentences_info.sentence_into);
    let user_dict=claims.user_dictionaries[claims.current_lang_index].id;
    let translate:ResultGptTranscript=state.gpt_api.send(query).await?;
    let sentences_info=sentences_info.into_inner();
    let dict=Dictionary_Sentence{id:0,user_dictionaries:user_dict,
        sentence_from:sentences_info.sentence_from,sentence_from_context:sentences_info.sentence_from_context,sentence_into:sentences_info.sentence_into,transcription_eng:translate.ipa,transcription_ukr:translate.ipa_ukr};
    let index= state.database.get_index_damp(user_dict).await?;
    state.database.add_dictionary_sentence( dict).await?;
    let sentence=state.database.get_dictionaries( user_dict, 0, 1).await?;
    if index==-1{
        state.database.add_index_damp( user_dict, sentence[0].id).await?;
    }
    Ok(HttpResponse::Ok().json(RequestResult{status:true}))
}
#[post("/dictionary/delete-item")]
pub async fn delete_dictionary_item(req:HttpRequest, sentences_info:web::Json<SentenceId>, state: web::Data<StateDb>) ->Result<HttpResponse>{
    let claims=req.extensions().get::<Claims>().ok_or(anyhow!("dont find cookie"))?.clone();
    let sentences_info=sentences_info.into_inner();
    let user_dict=claims.user_dictionaries[claims.current_lang_index].id;
    let index= state.database.get_index_damp(user_dict).await?;
    if index==sentences_info.id{
        let mut new_id=-1;
        let next_id= state.database.get_next_record_damp(user_dict, sentences_info.id).await?;
        if next_id==-1{
            let befor_id= state.database.get_befor_record_damp(user_dict, sentences_info.id).await?;
            if befor_id!=-1{
                new_id=befor_id;
            }
        }else{
            new_id=next_id;
        }
        if new_id==-1{
            state.database.delete_index_damp( user_dict).await?;
        }else{
            state.database.set_index_damp( user_dict, new_id).await?;
        }
    }
    state.database.delete_dictionary( sentences_info.clone()).await?;
    Ok(HttpResponse::Ok().json(RequestResult{status:true}))
}
#[post("/dictionary/set-index")]
pub async fn set_dictionary_index(req:HttpRequest, sentences_info:web::Json<SentenceId>, state: web::Data<StateDb>) ->Result<HttpResponse>{
    let claims=req.extensions().get::<Claims>().ok_or(anyhow!("dont find cookie"))?.clone();
    let user_dict=claims.user_dictionaries[claims.current_lang_index].id;
    state.database.set_index_damp( user_dict, sentences_info.into_inner().id).await?;
    Ok(HttpResponse::Ok().json(RequestResult{status:true}))
}
struct FileToDelete(PathBuf);

impl Drop for FileToDelete {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
#[get("/dictionary/export-current")]
pub async fn get_dictionary_from_index(req:HttpRequest, state: web::Data<StateDb>) ->Result<impl Responder>{
    let claims=req.extensions().get::<Claims>().ok_or(anyhow!("dont find cookie"))?.clone();
    let user_dict=claims.user_dictionaries[claims.current_lang_index].id;
    let lang_name=claims.user_dictionaries[claims.current_lang_index].language_name.clone();
    let index_dump= state.database.get_index_damp( user_dict).await?;
    let sentences= state.database.get_dictionaries_dump( user_dict, index_dump).await?;
    let string=generate_anki(user_dict,sentences,lang_name);
    let exe_path = env::current_exe().unwrap();
    let exe_dir = exe_path.parent().unwrap();
    let file_path = std::path::PathBuf::from(env!("HOME"))
        .as_path()
        .join(exe_dir)
        .join(string.clone());

    let file = actix_files::NamedFile::open_async(file_path.clone()).await.unwrap();
    let mut response = file.into_response(&req);

    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        header::HeaderValue::from_str(format!("attachment; filename=\"{}\"", string).as_str()).unwrap()
    );

    response.extensions_mut().insert(FileToDelete(PathBuf::from(file_path)));
    let first= state.database.get_dictionaries( user_dict, 0, 1).await?;
    state.database.set_index_damp( user_dict, first[0].id).await?;
    Ok(response)
}
#[get("/dictionary/export-all")]
pub async fn export_dictionary(req:HttpRequest, state: web::Data<StateDb>) ->Result<impl Responder>{
    let claims=req.extensions().get::<Claims>().ok_or(anyhow!("dont find cookie"))?.clone();
    let user_dict=claims.user_dictionaries[claims.current_lang_index].id;
    let lang_name=claims.user_dictionaries[claims.current_lang_index].language_name.clone();
    let sentences= state.database.get_dictionaries( user_dict, 0, 0).await?;
    let string=generate_anki(user_dict,sentences,lang_name);
    let exe_path = env::current_exe().unwrap();
    let exe_dir = exe_path.parent().unwrap();
    let file_path = std::path::PathBuf::from(env!("HOME"))
        .as_path()
        .join(exe_dir)
        .join(string.clone());

    let file = actix_files::NamedFile::open_async(file_path.clone()).await?;
    let mut response = file.into_response(&req);

    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        header::HeaderValue::from_str(format!("attachment; filename=\"{}\"", string).as_str()).unwrap()
    );

    response.extensions_mut().insert(FileToDelete(PathBuf::from(file_path)));
    let first= state.database.get_dictionaries(user_dict, 0, 1).await?;
    state.database.set_index_damp( user_dict, first[0].id).await?;
    Ok(response)
}