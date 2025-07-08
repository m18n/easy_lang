use actix_web::{get, HttpMessage, HttpRequest, HttpResponse, web};
use futures_util::future::join_all;
use ramhorns::Template;
use crate::base::{file_openString, get_nowtime_str};
use crate::jwt::Claims;
use crate::models::{LanguageSupported, DataBase};
use crate::render_temps::{CurrentLangTemplate, DictionaryTemplate, TranslateHistoryItemTemplate, TranslateHistoryTemplate, TranslateTemplate};
use crate::StateDb;
use anyhow::{anyhow, Result as AnyhowResult};
use crate::globals::Result;
// url controller: /view/userspace/***

#[get("/learn/main")]
pub async fn learn_page(req:HttpRequest, state: web::Data<StateDb>) ->Result<HttpResponse>{
    let cookie=req.extensions().get::<Claims>().ok_or(anyhow!("dont find cookie"))?.clone();
    let contents = file_openString("./easy_lang_web/learn_lang_main.html").await?;
    let template= CurrentLangTemplate {
        current_lang:cookie.user_dictionaries[cookie.current_lang_index].language_name.clone(),
        languages:cookie.user_dictionaries.clone(),
    };
    let tpl = Template::new(contents).unwrap();
    Ok(HttpResponse::Ok().content_type("text/html").body(tpl.render(&template)))
}
#[get("/dictionary")]
pub async fn dictionary_redirect(req:HttpRequest, state: web::Data<StateDb>) ->Result<HttpResponse>{
    let response = HttpResponse::Found()
        .insert_header((http::header::LOCATION,"/userspace/dictionary/p/1".to_string()))
        .finish();
    Ok(response)
}
#[get("/dictionary/p/{number_p}")]
pub async fn dictionary_page(req:HttpRequest, state: web::Data<StateDb>, number_p:web::Path<(i32)>) ->Result<HttpResponse>{
    let mut number_p=number_p.into_inner();
    number_p-=1;
    let cookie=req.extensions().get::<Claims>().ok_or(anyhow!("dont find cookie"))?.clone();
    let contents = file_openString("./easy_lang_web/dictionary.html").await?;
    let user_dict=cookie.user_dictionaries[cookie.current_lang_index].id;
    let dictionary= state.database.get_dictionaries( user_dict,
                                               number_p*10, 10).await?;
    let index_dump= state.database.get_index_damp(user_dict).await?;
    let template= DictionaryTemplate {
        current_lang:cookie.user_dictionaries[cookie.current_lang_index].language_name.clone(),
        languages:cookie.user_dictionaries.clone(),
        dictionary:dictionary,
        index_dump:index_dump
    };
    let tpl = Template::new(contents).unwrap();
    Ok(HttpResponse::Ok().content_type("text/html").body(tpl.render(&template)))
}
#[get("/translate/main")]
pub async fn translate_page(req:HttpRequest, state: web::Data<StateDb>) ->Result<HttpResponse>{
    let cookie=req.extensions().get::<Claims>().ok_or(anyhow!("dont find cookie"))?.clone();
    let langs= state.database.get_languages().await?;
    let contents = file_openString("./easy_lang_web/translate_main.html").await?;
    let template=TranslateTemplate{
        current_lang:cookie.user_dictionaries[cookie.current_lang_index].language_name.clone(),
        current_lang_id:cookie.user_dictionaries[cookie.current_lang_index].language_id,
        languages:cookie.user_dictionaries.clone(),
        langueges_supported:langs
    };
    let tpl = Template::new(contents).unwrap();
    Ok(HttpResponse::Ok().content_type("text/html").body(tpl.render(&template)))
}
#[get("/translate/history")]
pub async fn translate_history_redirect(req:HttpRequest, state: web::Data<StateDb>) ->Result<HttpResponse>{
    let cookie=req.extensions().get::<Claims>().ok_or(anyhow!("dont find cookie"))?.clone();
    let response = HttpResponse::Found()
        .insert_header((http::header::LOCATION, format!("/userspace/translate/history/{}/p/1",cookie.user_dictionaries[cookie.current_lang_index].language_name)))
        .finish();
    Ok(response)
}
#[get("/translate/history/{lang}/p/{number_p}")]
pub async fn translate_history_page(req:HttpRequest, state: web::Data<StateDb>, path:web::Path<(String, i32)>) ->Result<HttpResponse>{
    let (mut lang,mut num)=path.into_inner();
    num-=1;
    let cookie=req.extensions().get::<Claims>().ok_or(anyhow!("dont find cookie"))?.clone();
    let mut lang_id=LanguageSupported{id:-1,language_name:String::new()};
    if lang!="all"{
        lang_id= state.database.get_language_by_name(lang).await?;
    }
    let langs= state.database.get_languages().await?;
    let trans= state.database.get_translated_history( num*10, 10, cookie.user_id, lang_id.id).await?;
    let contents = file_openString("./easy_lang_web/translate_history.html").await?;
    let template=TranslateHistoryTemplate{
        current_lang:cookie.user_dictionaries[cookie.current_lang_index].language_name.clone(),
        current_lang_history_id:lang_id.id,
        languages:cookie.user_dictionaries.clone(),
        translate_history:trans,
        all_languages:langs
    };
    let tpl = Template::new(contents).unwrap();
    Ok(HttpResponse::Ok().content_type("text/html").body(tpl.render(&template)))
}
#[get("/translate/history/item/{id_item}")]
pub async fn translate_history_item(req:HttpRequest, state: web::Data<StateDb>, id_item_:web::Path<i32>) ->Result<HttpResponse>{
    let mut id_item=id_item_.into_inner();
    let cookie=req.extensions().get::<Claims>().ok_or(anyhow!("dont find cookie"))?.clone();
    let trans= state.database.get_translated_items( id_item, cookie.user_id).await?;
    let mut tasks=Vec::new();
    tasks.push(state.database.get_language(trans.lang_from_translated_id));
    tasks.push(state.database.get_language(trans.lang_into_translated_id));
    let results=join_all(tasks).await;
    let lang_from = results[0].as_ref().map_err(|e| anyhow::anyhow!("{}", e))?.language_name.clone();
    let lang_into = results[1].as_ref().map_err(|e| anyhow::anyhow!("{}", e))?.language_name.clone();

    let contents = file_openString("./easy_lang_web/translate_item.html").await?;
    let template=TranslateHistoryItemTemplate{
        current_lang:cookie.user_dictionaries[cookie.current_lang_index].language_name.clone(),
        languages:cookie.user_dictionaries.clone(),
        translate_history:trans,
        lang_from:lang_from,
        lang_into:lang_into
    };
    let tpl = Template::new(contents).unwrap();
    Ok(HttpResponse::Ok().content_type("text/html").body(tpl.render(&template)))
}