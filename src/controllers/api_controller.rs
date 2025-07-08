use actix_web::{HttpResponse, post, web};
use crate::controllers::object_of_controller::{AuthInfo, RequestResult};
use crate::cookie::create_cookie_auth;
use crate::jwt::{Claims};
use crate::models::{DataBase};
use crate::StateDb;
use crate::globals::Result;
// url controller: /api/***
#[post("/auth")]
pub async fn authenticate(auth_info:web::Json<AuthInfo>, state: web::Data<StateDb>) ->Result<HttpResponse>{
    let auth_obj=auth_info.into_inner();
    let res= state.database.check_auth(auth_obj.clone()).await?;
    if res!=-1 {
        let users_dictionaries= state.database.get_user_dictionaries( res).await?;
        let mut claims=Claims::new();
        claims.user_id=res;
        claims.user_name=auth_obj.user_name.clone();
        claims.admin=false;
        claims.user_dictionaries=users_dictionaries;
        claims.current_lang_index=0;
        let cookie=create_cookie_auth(claims.clone());
        let mut respon = HttpResponse::Ok().cookie(cookie).json(RequestResult { status: true });
        Ok(respon)
    }else{
        let mut respon = HttpResponse::Ok().json(RequestResult { status: false });
        Ok(respon)
    }

}
