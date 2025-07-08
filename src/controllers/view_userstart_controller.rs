use actix_web::{get, HttpResponse, web};
use ramhorns::Template;
use crate::base::file_openString;
use crate::models::{ DataBase};
use crate::render_temps::InitTemplate;
use crate::StateDb;
use anyhow::{anyhow, Result as AnyhowResult};
use crate::globals::Result;
// url controller: /view/userstart/***

#[get("/init-dictionaries")]
pub async fn init_dictionaries_page(state: web::Data<StateDb>) ->Result<HttpResponse>{
    let dictionaries= state.database.get_languages().await?;
    let lang_levels= state.database.get_languages_levels().await?;
    let contents = file_openString("./easy_lang_web/init_dictionaries.html").await?;
    let template=InitTemplate{
        languages:dictionaries,
        languages_levels:lang_levels
    };
    let tpl = Template::new(contents).unwrap();
    Ok(HttpResponse::Ok().content_type("text/html").body(tpl.render(&template)))
}