# Easy Lang

**Easy Lang** is a web server written in Rust that makes learning English and other languages easier. Using powerful technologies such as the GPT-4 API for translation and the Google API for voicing, Easy Lang helps users better understand and remember new words and phrases.
It is primarily designed to support Ukrainian users, helping them learn languages in a way that is intuitive and culturally relevant.

## Features

- **Smart Translation:** Uses GPT-4 to provide contextual and accurate translation.
- **Voiceover:** Translated phrases can be heard using the Google API to help improve pronunciation.
- **History of translations:** Each user has a personal history of translations, which allows you to conveniently view and repeat previously translated words and phrases.
- **Dictionary:** Ability to add your own words and phrases to repeat. This dictionary becomes your personal study kit.
- **Export to Anki:** The function to export words and phrases to an `.apkg` file for use in the popular program Anki, which allows you to effectively repeat the material.

## Technical details

- **Programming language:** Rust
- **Translation API:** GPT-4 API
- **Voice API:** Google API
- **Data storage:** Translation history is stored in the database for each user separately.

## Usage

After successfully starting the server, open your browser and go to `http://localhost:3002`. Register or log in to your account to start using Easy Lang.
