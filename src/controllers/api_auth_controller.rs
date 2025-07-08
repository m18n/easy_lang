use actix_web::{get, HttpMessage, HttpRequest, HttpResponse, post, web};
use crate::base::{get_nowtime_str};
use crate::controllers::object_of_controller::{DictionariesInfo, RequestResult};
use crate::cookie::create_cookie_auth;
use crate::jwt::{Claims};
use crate::models::{DataBase};
use crate::StateDb;
use anyhow::{anyhow, Result as AnyhowResult};
use crate::globals::Result;
// url controller: /api/userstart/***
#[get("/test")]
pub async fn m_test()->Result<HttpResponse>{

    Ok(HttpResponse::Ok().content_type("text/html").body("Hello"))
}
#[post("/set-dictionaries")]
pub async fn set_dictionaries(req:HttpRequest, dictionaries_id:web::Json<DictionariesInfo>, state: web::Data<StateDb>) ->Result<HttpResponse>{
    if let Some(claims) = req.extensions().get::<Claims>(){
         state.database.set_dictionaries( dictionaries_id.into_inner(), claims.user_id).await?;
        let user_dictionaries= state.database.get_user_dictionaries(claims.user_id).await?;
        let my_claims=Claims{
            user_dictionaries:user_dictionaries,
            current_lang_index:0,
            ..claims.clone()
        };
        let cookie=create_cookie_auth(my_claims.clone());
        let mut respon = HttpResponse::Ok().cookie(cookie).json(RequestResult { status: true });
        Ok(respon)
    }else{
        Err(anyhow!("Client dont have auth").into())
    }
}