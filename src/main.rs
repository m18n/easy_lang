mod models;
mod globals;
mod base;
mod controllers;
mod no_cache_middleware;

mod jwt;
mod check_user_middleware;
mod check_auth_middleware;
mod render_temps;
mod check_auth_api_middleware;
mod check_user_api_middleware;
mod gpt_module;
mod translate_module;
mod google_module;
mod cookie;
mod generate_anki;
mod migrations;

use std::env;
use std::sync::Arc;
use actix_web::{App, HttpServer, web};
use actix_files as fs;
use async_openai::Client;
use async_openai::config::OpenAIConfig;
use deepl::DeepLApi;
use no_cache_middleware::NoCache;
use tokio::sync::Mutex;
use crate::check_auth_api_middleware::CheckAuthApi;
use crate::check_auth_middleware::CheckAuth;
use crate::check_user_middleware::CheckUser;
use crate::check_user_api_middleware::CheckUserApi;
use crate::controllers::{api_auth_controller, api_controller, api_service_controller, api_user_controller, settings_controller, view_controller, view_userspace_controller, view_userstart_controller};
use crate::gpt_module::GptModule;
use crate::models::{DataBase};
use crate::translate_module::DeeplModule;
use dotenv::dotenv;
use actix_cors::Cors;
use crate::google_module::GoogleModule;
use anyhow::Result;
use chrono::Local;
use tracing::{info, Level};
use tracing_subscriber::{self, fmt::format::FmtSpan};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use crate::controllers::view_controller::error_page;

pub struct StateDb{
    pub database:DataBase,
    deepl_api:DeeplModule,
    gpt_api:GptModule,
    google_module:GoogleModule
}

#[actix_web::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix(format!("easy_lang_{}", Local::now().format("%Y%m%d_%H%M%S")))
        .filename_suffix(".txt")
        .build("logs")
        .expect("Failed to create log appender");

    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env()
            .add_directive(Level::INFO.into())
        )
        .with_span_events(FmtSpan::CLOSE)
        .init();


    info!("Initializing application");
    let state=web::Data::new(StateDb{
        google_module:GoogleModule::new(env::var("GOOGLE_API").unwrap()),
        database:DataBase::new(env::var("DATABASE_URL").unwrap()).await?,
        deepl_api:DeeplModule::new(env::var("DEEPL_API").unwrap()),
        gpt_api:GptModule::new(env::var("GPT_API").unwrap()).await
    });

    HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .wrap(cors)
            .app_data(web::Data::clone(&state))
            .default_service(web::route().to(settings_controller::redirect_to_login))
            .wrap(NoCache)
            .service(error_page)
            .service(fs::Files::new("/public", "./easy_lang_web").show_files_listing())

                    .service(view_controller::login_page)
                    .service(
                        web::scope("/userstart")
                            .wrap(CheckAuth)
                            .service(view_userstart_controller::init_dictionaries_page)
                    )
                    .service(
                        web::scope("/userspace")
                            .wrap(CheckUser)
                            .service(view_userspace_controller::learn_page)
                            .service(view_userspace_controller::dictionary_redirect)
                            .service(view_userspace_controller::dictionary_page)
                            .service(view_userspace_controller::translate_page)
                            .service(view_userspace_controller::translate_history_redirect)
                            .service(view_userspace_controller::translate_history_page)
                            .service(view_userspace_controller::translate_history_item)
                    )



            .service(
                web::scope("/api")
                    .service(
                        web::scope("/service")
                            .service(api_service_controller::text_to_audio)
                            .service(api_service_controller::check_answer)
                    )
                    .service(api_controller::authenticate)
                    .service(
                        web::scope("/userspace")
                            .wrap(CheckUserApi)
                            .service(api_user_controller::m_test)
                            .service(api_user_controller::set_current_dictionary)
                            .service(api_user_controller::deepl_translate)
                            .service(api_user_controller::gpt_translate_full_formal)
                            .service(api_user_controller::gpt_translate_short_formal)
                            .service(api_user_controller::gpt_translate_full_conversational)
                            .service(api_user_controller::gpt_translate_short_conversational)
                            .service(api_user_controller::save_translation)
                            .service(api_user_controller::delete_translation)
                            .service(api_user_controller::logout)
                            .service(api_user_controller::add_dictionary_sentence)
                            .service(api_user_controller::delete_dictionary_item)
                            .service(api_user_controller::set_dictionary_index)
                            .service(api_user_controller::get_dictionary_from_index)
                            .service(api_user_controller::export_dictionary)
                    )
                    .service(
                        web::scope("/userstart")
                            .wrap(CheckAuthApi)
                            .service(api_auth_controller::m_test)
                            .service(api_auth_controller::set_dictionaries)
                    )
            )
    })
        .bind(("0.0.0.0", 3002))?
        .run()
        .await?;
    Ok(())
}