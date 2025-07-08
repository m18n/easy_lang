use actix_web::{HttpResponse, ResponseError};
use futures_util::future::join_all;
use http::StatusCode;
use ramhorns::Content;
use serde::{Deserialize, Serialize};
use sqlx::{Error, PgPool, query};
use sqlx::FromRow;
use crate::controllers::object_of_controller::{AuthInfo, DictionariesInfo};
use anyhow::{anyhow, Result};
use tracing::info;
use crate::migrations::{get_version, set_version, MIGRATIONS};

#[derive(Debug, Serialize, Deserialize,sqlx::FromRow)]
pub struct User{
    id:i32,
    user_name:String,
    password:String,
}
#[derive(Debug, Serialize, Deserialize,sqlx::FromRow,Clone,Content)]
pub struct UserDictionary{
    pub id:i32,
    pub language_name:String,
    pub language_id:i32,
    pub language_level:String,
    pub language_level_id:i32
}
#[derive(Debug, Serialize, Deserialize,sqlx::FromRow,Clone,Content)]
pub struct LanguageSupported{
    pub id:i32,
    pub language_name:String
}
#[derive(Debug, Serialize, Deserialize,sqlx::FromRow,Clone,Content)]
pub struct LanguagesLevels{
    pub id:i32,
    pub level_name:String
}

#[derive(Debug, Serialize, Deserialize, FromRow,Clone,PartialEq,Content)]
pub struct Translated{
    pub id:i32,
    pub lang_from_translated_id:i32,
    pub lang_into_translated_id:i32,
    pub translated_text:String,
    pub context_text:String,
    pub deepl_translated:String,
    pub deepl_check_deepl:String,
    pub speak_gpt_translated:String,
    pub speak_deepl_check_gpt_translated:String,
    pub speak_explanation_gpt:String,
    pub formal_gpt_translated:String,
    pub formal_deepl_check_gpt_translated:String,
    pub formal_explanation_gpt:String,
    pub is_full:bool
}
#[derive(Debug, Serialize, Deserialize, FromRow,Clone,PartialEq,Content)]
pub struct TranslatedId{
    pub id:i32,
}
#[derive(Debug, Serialize, Deserialize, FromRow,Clone,PartialEq,Content)]
pub struct SentenceId{
    pub id:i32,
}
#[derive(Debug, Serialize, Deserialize, FromRow,Clone,PartialEq,Content)]
pub struct Dictionary_Sentence{
    pub id:i32,
    pub user_dictionaries:i32,
    pub sentence_from:String,
    pub sentence_from_context:String,
    pub sentence_into:String,
    pub transcription_eng:String,
    pub transcription_ukr:String
}
impl Translated {
    pub fn new()->Self{
        Self{id:-1,lang_from_translated_id:-1,lang_into_translated_id:-1,translated_text:String::new()
            ,context_text:String::new(),deepl_translated:String::new(),deepl_check_deepl:String::new(),speak_gpt_translated:String::new(),
            speak_deepl_check_gpt_translated:String::new(),speak_explanation_gpt:String::new(),formal_gpt_translated:String::new(),
            formal_deepl_check_gpt_translated:String::new(),formal_explanation_gpt:String::new(),is_full:false}
    }
}

//test
pub struct DataBase {
    pub pool:PgPool,
}
impl DataBase {
    pub  async  fn new(db_url:String)->Result<DataBase>{
        let pool=PgPool::connect(&db_url).await?;
        let db=DataBase { pool };
        db.run_migrations().await?;
        Ok(db)
    }
    async fn run_migrations(&self) -> Result<()> {
        info!("Running migrations");
        let create_version_table_commands = vec![
            r#"
            CREATE TABLE IF NOT EXISTS version_easy_lang(
                id SERIAL PRIMARY KEY,
                version_number INTEGER NOT NULL
            )
            "#,
            r#"
            INSERT INTO version_easy_lang (version_number)
            SELECT 0
            WHERE NOT EXISTS (SELECT 1 FROM version_easy_lang)
            "#
        ];

        for command in create_version_table_commands {
            sqlx::query(command).execute(&self.pool).await?;
        }

        let current_version: i32 = sqlx::query_scalar("SELECT version_number FROM version_easy_lang LIMIT 1")
            .fetch_one(&self.pool)
            .await?;


        for migration in MIGRATIONS.iter() {
            if migration.from >= current_version {
                info!("Applying migration from version {} to {}", migration.from, migration.to);

                let mut tx = self.pool.begin().await?;

                let mut inside_dollar_quote = false;
                let mut command_buffer = String::new();

                for line in migration.script.lines() {
                    let trimmed_line = line.trim();

                    if trimmed_line.contains("$$") {
                        inside_dollar_quote = !inside_dollar_quote;
                    }

                    command_buffer.push_str(line);
                    command_buffer.push('\n');

                    if !inside_dollar_quote && trimmed_line.ends_with(';') {
                        sqlx::query(&command_buffer).execute(&mut *tx).await?;
                        command_buffer.clear();
                    }
                }

                if !command_buffer.trim().is_empty() {
                    sqlx::query(&command_buffer).execute(&mut *tx).await?;
                }

                sqlx::query("UPDATE version_easy_lang SET version_number = $1")
                    .bind(migration.to)
                    .execute(&mut *tx)
                    .await?;

                tx.commit().await?;

                info!("Migration to version {} completed", migration.to);
            }
        }

        info!("Migrations completed");
        set_version(MIGRATIONS[MIGRATIONS.len()-1].to).await;
        info!("PROGRAM VERSION: {}",get_version().await);
        Ok(())
    }
    pub async fn check_auth(&self, auth_info:AuthInfo) ->Result<i32>{
        let users:Vec<User>= sqlx::query_as("SELECT * FROM users WHERE user_name=$1;")
            .bind(auth_info.user_name)
            .fetch_all(&self.pool)
            .await?;
        if !users.is_empty() && users[0].password==auth_info.password{
            Ok(users[0].id)
        }else{
            Ok(-1)
        }
    }
    pub async fn get_user_dictionaries(&self, user_id:i32) ->Result<Vec<UserDictionary>>{


        let user_dictionary:Vec<UserDictionary>= sqlx::query_as("SELECT ud.id, ls.language_name, ls.id AS language_id, lv.id AS language_level_id , lv.level_name AS language_level
        FROM user_dictionaries AS ud
        JOIN languages_supported AS ls ON ud.language_id = ls.id
        JOIN language_levels AS lv ON ud.language_level = lv.id
        WHERE ud.user_id = $1;")
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?;
       Ok(user_dictionary)
    }
    pub async fn get_languages(&self) ->Result<Vec<LanguageSupported>>{
        let languages_supported:Vec<LanguageSupported>= sqlx::query_as("SELECT * FROM languages_supported")
            .fetch_all(&self.pool)
            .await?;
        Ok(languages_supported)
    }

    pub async fn get_languages_levels(&self) ->Result<Vec<LanguagesLevels>>{

        let languages_supported:Vec<LanguagesLevels>= sqlx::query_as("SELECT * FROM language_levels")
            .fetch_all(&self.pool)
            .await?;
        Ok(languages_supported)
    }
    pub async fn get_language(&self, id_lang:i32) ->Result<LanguageSupported>{
        let languages_supported:Vec<LanguageSupported>= sqlx::query_as("SELECT * FROM languages_supported WHERE id=$1")
            .bind(id_lang)
            .fetch_all(&self.pool)
            .await?;
        if languages_supported.is_empty(){
            return Err(anyhow!("couldn't find the language"));
        }
        Ok(languages_supported[0].clone())
    }
    pub async fn get_language_by_name(&self, lang_name:String) ->Result<LanguageSupported>{
        let languages_supported:Vec<LanguageSupported>= sqlx::query_as("SELECT * FROM languages_supported WHERE language_name=$1;")
            .bind(lang_name)
            .fetch_all(&self.pool)
            .await?;
        if languages_supported.is_empty(){
            return Err(anyhow!("couldn't find the language"));
        }
        Ok(languages_supported[0].clone())
    }
    pub async fn get_translated_history(&self, start_element:i32, size_element:i32, user_id:i32, lang_id:i32) ->Result<Vec<Translated>>{

        let mut query=String::new();
        if lang_id!=-1 {
            query = format!("SELECT * FROM translation_history WHERE user_id={} AND lang_into_translated_id={} ORDER BY id DESC LIMIT {} OFFSET {} ;"
                                , user_id, lang_id, size_element, start_element);
        }else{
            query = format!("SELECT * FROM translation_history WHERE user_id={} ORDER BY id DESC LIMIT {} OFFSET {} ;"
                                , user_id, size_element, start_element);
        }
        let translated:Vec<Translated>= sqlx::query_as(query.as_str())
            .fetch_all(&self.pool)
            .await?;
        Ok(translated)
    }
    pub async fn get_dictionaries(&self, dict_id: i32, start_element: i32, size_element: i32) -> Result<Vec<Dictionary_Sentence>> {
        let mut query = format!(
            "SELECT * FROM anki_sentences WHERE user_dictionaries={} ORDER BY id DESC",
            dict_id
        );

        if size_element > 0 {
            query.push_str(&format!(" LIMIT {}", size_element));
        }

        if start_element > 0 {
            query.push_str(&format!(" OFFSET {}", start_element));
        }

        query.push(';');

        let dict: Vec<Dictionary_Sentence> = sqlx::query_as(query.as_str())
            .fetch_all(&self.pool)
            .await?;

        Ok(dict)
    }
    pub async fn get_dictionaries_dump(&self, dict_id:i32, index_dump:i32) ->Result<Vec<Dictionary_Sentence>>{
        let dict:Vec<Dictionary_Sentence>= sqlx::query_as("SELECT * FROM anki_sentences WHERE user_dictionaries=$1 AND id>$2 ORDER BY id ASC;")
            .bind(dict_id)
            .bind(index_dump)
            .fetch_all(&self.pool)
            .await?;
        Ok(dict)
    }
    pub async fn get_translated_items(&self, id_item:i32, user_id:i32) ->Result<Translated>{
        let query=format!("SELECT * FROM translation_history WHERE user_id={} AND id={};",user_id,id_item);
        let translated:Vec<Translated>= sqlx::query_as(query.as_str())
            .fetch_all(&self.pool)
            .await?;
        if translated.is_empty(){
            return Err(anyhow!("couldn't find the history"));
        }
        Ok(translated[0].clone())
    }

    pub async fn set_dictionaries(&self, dictionaries_info: DictionariesInfo, user_id: i32) -> Result<bool> {
        let mut tasks_array = Vec::new();

        for i in 0..dictionaries_info.dictionaries_ids.len() {
            // Create a closure that captures the values
            let lang_id = dictionaries_info.dictionaries_ids[i];
            let level_id = dictionaries_info.dictionaries_level_ids[i];
            let uid = user_id;
            let pool = self.pool.clone();

            // Create a task that uses parameter binding
            let task = async move {
                let result = sqlx::query("INSERT INTO user_dictionaries (user_id, language_id, language_level) VALUES ($1, $2, $3)")
                    .bind(uid)
                    .bind(lang_id)
                    .bind(level_id)
                    .execute(&pool)
                    .await?;

                if result.rows_affected() == 0 {
                    return Err(anyhow::anyhow!("Failed to insert dictionary with language ID {} and level {}", lang_id, level_id));
                }

                Ok::<_, anyhow::Error>(())
            };

            tasks_array.push(task);
        }

        let results = join_all(tasks_array).await;
        for res in results {
            res?;
        }

        Ok(true)
    }
    pub async fn save_translate(&self, translated_info: Translated, user_id: i32) -> Result<bool> {
        let res = sqlx::query(
            "INSERT INTO translation_history (
            lang_from_translated_id, lang_into_translated_id, translated_text,
            context_text, deepl_translated, deepl_check_deepl, speak_gpt_translated,
            speak_deepl_check_gpt_translated, speak_explanation_gpt,
            formal_gpt_translated, formal_deepl_check_gpt_translated, formal_explanation_gpt,
            is_full, user_id
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)"
        )
            .bind(translated_info.lang_from_translated_id)
            .bind(translated_info.lang_into_translated_id)
            .bind(&translated_info.translated_text)
            .bind(&translated_info.context_text)
            .bind(&translated_info.deepl_translated)
            .bind(&translated_info.deepl_check_deepl)
            .bind(&translated_info.speak_gpt_translated)
            .bind(&translated_info.speak_deepl_check_gpt_translated)
            .bind(&translated_info.speak_explanation_gpt)
            .bind(&translated_info.formal_gpt_translated)
            .bind(&translated_info.formal_deepl_check_gpt_translated)
            .bind(&translated_info.formal_explanation_gpt)
            .bind(translated_info.is_full)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(true)
    }
    pub async fn delete_translated(&self, translated_info: TranslatedId, user_id: i32) -> Result<bool> {
        let result = sqlx::query("DELETE FROM translation_history WHERE user_id = $1 AND id = $2")
            .bind(user_id)
            .bind(translated_info.id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("No translation history found with the specified ID or you don't have permission to delete it"));
        }

        Ok(true)
    }
    pub async fn delete_dictionary(&self, sentence_id: SentenceId) -> Result<bool> {
        let result = sqlx::query("DELETE FROM anki_sentences WHERE id = $1")
            .bind(sentence_id.id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("No dictionary entry found with the specified ID"));
        }

        Ok(true)
    }
    pub async fn set_index_damp(&self, user_dictionary: i32, index: i32) -> Result<bool> {
        let result = sqlx::query("UPDATE anki_dump SET sentences_id = $1 WHERE user_dictionaries = $2")
            .bind(index)
            .bind(user_dictionary)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("No anki dump found for the specified user dictionary"));
        }

        Ok(true)
    }
    pub async fn delete_index_damp(&self, user_dictionary: i32) -> Result<bool> {
        let result = sqlx::query("DELETE FROM anki_dump WHERE user_dictionaries = $1")
            .bind(user_dictionary)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("No anki dump found for the specified user dictionary"));
        }

        Ok(true)
    }
    pub async fn get_next_record_damp(&self, user_dictionary: i32, id_dump: i32) -> Result<i32> {
        let sentences_ids: Vec<i32> = sqlx::query_scalar("SELECT id FROM anki_sentences WHERE id < $1 AND user_dictionaries = $2 ORDER BY id DESC LIMIT 1")
            .bind(id_dump)
            .bind(user_dictionary)
            .fetch_all(&self.pool)
            .await?;

        if sentences_ids.is_empty() {
            Ok(-1)
        } else {
            Ok(sentences_ids[0])
        }
    }
    pub async fn get_befor_record_damp(&self, user_dictionary: i32, id_dump: i32) -> Result<i32> {
        let sentences_ids: Vec<i32> = sqlx::query_scalar("SELECT id FROM anki_sentences WHERE id > $1 AND user_dictionaries = $2 ORDER BY id ASC LIMIT 1")
            .bind(id_dump)
            .bind(user_dictionary)
            .fetch_all(&self.pool)
            .await?;

        if sentences_ids.is_empty() {
            Ok(-1)
        } else {
            Ok(sentences_ids[0])
        }
    }
    pub async fn get_index_damp(&self, user_dictionary: i32) -> Result<i32> {
        let sentences_ids: Vec<i32> = sqlx::query_scalar("SELECT sentences_id FROM anki_dump WHERE user_dictionaries = $1")
            .bind(user_dictionary)
            .fetch_all(&self.pool)
            .await?;

        if sentences_ids.is_empty() {
            Ok(-1)
        } else {
            Ok(sentences_ids[0])
        }
    }

    pub async fn add_index_damp(&self, user_dictionary: i32, sentence_id: i32) -> Result<bool> {
        let result = sqlx::query("INSERT INTO anki_dump (user_dictionaries, sentences_id) VALUES ($1, $2)")
            .bind(user_dictionary)
            .bind(sentence_id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("Failed to add index dump record"));
        }

        Ok(true)
    }

    pub async fn add_dictionary_sentence(&self, sentence_info: Dictionary_Sentence) -> Result<bool> {
        let result = sqlx::query(
            "INSERT INTO anki_sentences (user_dictionaries, sentence_from, sentence_from_context, sentence_into, transcription_eng, transcription_ukr)
         VALUES ($1, $2, $3, $4, $5, $6)"
        )
            .bind(sentence_info.user_dictionaries)
            .bind(&sentence_info.sentence_from)
            .bind(&sentence_info.sentence_from_context)
            .bind(&sentence_info.sentence_into)
            .bind(&sentence_info.transcription_eng)
            .bind(&sentence_info.transcription_ukr)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("Failed to add dictionary sentence"));
        }

        Ok(true)
    }
}