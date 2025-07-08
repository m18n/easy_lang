use actix_web::{get, HttpResponse, post, web};
use actix_web::web::Json;
use crate::controllers::object_of_controller::{RequestResult, ResultAnkiGpt, ResultGptCheck, ResultGptPuzzle, SentencesLang, TextToSpeach};
use crate::google_module::GoogleModule;
use crate::gpt_module::GptModule;
use anyhow::{anyhow, Result as AnyhowResult};
use crate::globals::Result;
use futures_util::SinkExt;
use crate::StateDb;
// url controller: /api/service/***


#[post("/text-to-audio")]
pub async fn text_to_audio(text_:web::Json<TextToSpeach>, state: web::Data<StateDb>) ->Result<HttpResponse>{
    let bytes=state.google_module.text_to_speach(text_.text.clone(),text_.name_lang.clone()).await?;
    Ok(HttpResponse::Ok()
        .content_type("audio/mpeg").body(web::Bytes::from(bytes)))

}
#[post("/check-answer")]
pub async fn check_answer(text_:web::Json<SentencesLang>, state: web::Data<StateDb>) ->Result<Json<ResultAnkiGpt>>{
    let text=text_.into_inner();
    let query=format!(r#" Я тобі надам українське речення з контекстом та {}.
Українське речення: "{}"
Контекст українського речення: "{}"
{} речення: "{}"
Ти маєш надати у відповді 2 параметри.
Перший це "assessment" на скільки хорошиї переклад з українського в тому контексті на {} від 0 до 100, це звісно приблизно.
Другий це "correct_translation" відкорегований переклад мого речення на англійську мову.
        Відповідь надай в JSON. У форматі об'єкту:
        {{
            "assessment":,
            "correct_translation":"",
        }}
        "#,text.lang_name,text.sentence_from,text.sentence_from_context,text.lang_name,text.sentence_into,text.lang_name);
    let gpt_translation:ResultGptCheck=state.gpt_api.send(query).await?;
    let words: Vec<String> = gpt_translation.correct_translation.split_whitespace().map(|s| s.to_string())
        .collect();
    let size_words=words.len()*2;
    let query=format!(r#" Я тобі надам українське речення з контекстом та {}.
Українське речення: "{}"
Контекст українського речення: "{}"
{} речення: "{}"
Ти маєш надати у відповдь 1 параметр.
Перший — "words_puzzle", я хочу зібрати {} речень як пазли, для цього мені потрібно, щоб ти згенерував масив з {} слів, які б мене заплутали, тільки не згадуйте ті, які вже є в реченні для {} речень.
        Відповідь надай в JSON. У форматі об'єкту:
        {{
            "words_puzzle":[""],
        }}
        "#, text.lang_name, text.sentence_from, text.sentence_from_context, text.lang_name, gpt_translation.correct_translation, text.lang_name, size_words, text.lang_name);
    let gpt_puzzle:ResultGptPuzzle=state.gpt_api.send(query).await?;
    let res_anki=ResultAnkiGpt{assessment: gpt_translation.assessment,correct_translation: gpt_translation.correct_translation
        ,words_puzzle:gpt_puzzle.words_puzzle,words_correct:words};
    Ok(Json(res_anki))
}