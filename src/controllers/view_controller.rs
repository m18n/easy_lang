use actix_web::{get, HttpResponse};
use crate::base::file_openString;
use crate::models::MyError;

#[get("/login")]
pub async fn m_login()->Result<HttpResponse, MyError>{
    let contents = file_openString("./easy_english_web/login.html").await?;
    Ok(HttpResponse::Ok().content_type("text/html").body(contents))
}

#[get("/main")]
pub async fn m_main()->Result<HttpResponse, MyError>{
    let contents = file_openString("./easy_english_web/main.html").await?;
    Ok(HttpResponse::Ok().content_type("text/html").body(contents))
}

#[get("/initdictionaries")]
pub async fn m_init_dictionaries()->Result<HttpResponse, MyError>{
    let contents = file_openString("./easy_english_web/init_dictionaries.html").await?;
    Ok(HttpResponse::Ok().content_type("text/html").body(contents))
}