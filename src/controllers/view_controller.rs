// controller for all visible pages
use actix_web::{get, HttpMessage, HttpResponse};
use crate::base::{file_openString};
use crate::globals::Result;
// url controller: /view/***
#[get("/login")]
pub async fn login_page() ->Result<HttpResponse>{
    let contents = file_openString("./easy_lang_web/login.html").await?;
    Ok(HttpResponse::Ok().content_type("text/html").body(contents))
}


#[get("/error")]
pub async fn error_page() ->Result<HttpResponse>{
    let contents = file_openString("./easy_lang_web/error_web_site.html").await?;
    Ok(HttpResponse::Ok().content_type("text/html").body(contents))
}

