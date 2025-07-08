use lazy_static::lazy_static;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Migration {
    pub from: i32,
    pub to: i32,
    pub script: &'static str,
}



lazy_static! {
    static ref VERSION: Mutex<i32> = Mutex::new(0);
}

pub async  fn get_version() -> i32 {
    let version = VERSION.lock().await;
    *version
}

pub async  fn set_version(new_version: i32) {
    let mut version = VERSION.lock().await;
    *version = new_version;
}
lazy_static! {
    pub static ref MIGRATIONS: Vec<Migration> = vec![
        Migration {
            from: 0,
            to: 1,
            script: r#"
                CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    user_name VARCHAR(255) NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL
);

CREATE TABLE IF NOT EXISTS languages_supported (
    id SERIAL PRIMARY KEY,
    language_name VARCHAR(50) NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS language_levels (
    id SERIAL PRIMARY KEY,
    level_name VARCHAR(20) NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS user_dictionaries (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    language_id INTEGER NOT NULL REFERENCES languages_supported(id) ON DELETE CASCADE,
    language_level INTEGER NOT NULL REFERENCES language_levels(id) ON DELETE CASCADE,
    UNIQUE(user_id, language_id)
);

CREATE TABLE IF NOT EXISTS translation_history (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    lang_from_translated_id INTEGER NOT NULL REFERENCES languages_supported(id),
    lang_into_translated_id INTEGER NOT NULL REFERENCES languages_supported(id),
    translated_text TEXT NOT NULL,
    context_text TEXT,
    deepl_translated TEXT,
    deepl_check_deepl TEXT,
    speak_gpt_translated TEXT,
    speak_deepl_check_gpt_translated TEXT,
    speak_explanation_gpt TEXT,
    formal_gpt_translated TEXT,
    formal_deepl_check_gpt_translated TEXT,
    formal_explanation_gpt TEXT,
    is_full BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS anki_sentences (
    id SERIAL PRIMARY KEY,
    user_dictionaries INTEGER NOT NULL REFERENCES user_dictionaries(id) ON DELETE CASCADE,
    sentence_from TEXT NOT NULL,
    sentence_from_context TEXT,
    sentence_into TEXT NOT NULL,
    transcription_eng TEXT,
    transcription_ukr TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS anki_dump (
    id SERIAL PRIMARY KEY,
    user_dictionaries INTEGER NOT NULL REFERENCES user_dictionaries(id) ON DELETE CASCADE,
    sentences_id INTEGER NOT NULL,
    UNIQUE(user_dictionaries)
);

-- Insert some default language levels if they don't exist
INSERT INTO language_levels (level_name)
SELECT 'A1' WHERE NOT EXISTS (SELECT 1 FROM language_levels WHERE level_name = 'A1');

INSERT INTO language_levels (level_name)
SELECT 'A2' WHERE NOT EXISTS (SELECT 1 FROM language_levels WHERE level_name = 'A2');

INSERT INTO language_levels (level_name)
SELECT 'B1' WHERE NOT EXISTS (SELECT 1 FROM language_levels WHERE level_name = 'B1');

INSERT INTO language_levels (level_name)
SELECT 'B2' WHERE NOT EXISTS (SELECT 1 FROM language_levels WHERE level_name = 'B2');

INSERT INTO language_levels (level_name)
SELECT 'C1' WHERE NOT EXISTS (SELECT 1 FROM language_levels WHERE level_name = 'C1');

INSERT INTO language_levels (level_name)
SELECT 'C2' WHERE NOT EXISTS (SELECT 1 FROM language_levels WHERE level_name = 'C2');

-- Insert some common languages if they don't exist
INSERT INTO languages_supported (language_name)
SELECT 'English' WHERE NOT EXISTS (SELECT 1 FROM languages_supported WHERE language_name = 'English');

INSERT INTO languages_supported (language_name)
SELECT 'Ukrainian' WHERE NOT EXISTS (SELECT 1 FROM languages_supported WHERE language_name = 'Ukrainian');

INSERT INTO languages_supported (language_name)
SELECT 'German' WHERE NOT EXISTS (SELECT 1 FROM languages_supported WHERE language_name = 'German');

INSERT INTO languages_supported (language_name)
SELECT 'French' WHERE NOT EXISTS (SELECT 1 FROM languages_supported WHERE language_name = 'French');

INSERT INTO languages_supported (language_name)
SELECT 'Spanish' WHERE NOT EXISTS (SELECT 1 FROM languages_supported WHERE language_name = 'Spanish');
            "#,
        },
        
    ];
}