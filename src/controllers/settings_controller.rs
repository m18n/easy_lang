//controller for settings web server
use actix_web::{get, HttpResponse, Responder, web};
use actix_web::http::header;

// url controller: /settings/***

pub async fn redirect_to_login() -> impl Responder{
    HttpResponse::Found()
        .insert_header((header::LOCATION, "/login"))
        .finish()
}
