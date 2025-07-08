use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use anyhow::Error as AnyhowError;
use std::fmt;
use tracing::log::error;

// Створюємо обгортку навколо anyhow::Error
pub struct AppError(pub AnyhowError);

// Оголошуємо тип Result, який можна використовувати в обробниках
pub type Result<T> = std::result::Result<T, AppError>;

// Автоматичне перетворення з anyhow::Error в AppError
impl From<AnyhowError> for AppError {
    fn from(error: AnyhowError) -> Self {
        AppError(error)
    }
}

// Додайте конвертацію стандартних помилок вводу-виводу
impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        AppError(anyhow::Error::new(error))
    }
}

// Якщо використовуєте serde_json
impl From<serde_json::Error> for AppError {
    fn from(error: serde_json::Error) -> Self {
        AppError(anyhow::Error::new(error))
    }
}

// Якщо використовуєте reqwest
impl From<reqwest::Error> for AppError {
    fn from(error: reqwest::Error) -> Self {
        AppError(anyhow::Error::new(error))
    }
}
// Реалізація Display для помилки
impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Реалізація Debug для помилки
impl fmt::Debug for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

// Тепер реалізуємо ResponseError для нашого власного типу
impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        // Логуємо помилку перед редіректом
        error!("Application error: {:?}", self.0);

        // Перенаправляємо на сторінку з помилкою
        HttpResponse::Found()
            .append_header(("Location", "/error"))
            .finish()
    }

    fn status_code(&self) -> StatusCode {
        // Встановлюємо статус код для редіректу
        StatusCode::FOUND // 302 Found для редіректу
    }
}