use actix_web::HttpResponse;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("initialization error: {0}")]
    InitializationError(String),

    #[error("connection error: {0}")]
    ConnectionError(String),

    #[error("serialization error: {0}")]
    SerializationError(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("session not found")]
    NotFound,
}

impl Error {
    pub fn to_http_response(&self) -> HttpResponse {
        match self {
            Error::InitializationError(msg) => {
                HttpResponse::InternalServerError().body(msg.clone())
            }
            Error::ConnectionError(msg) => HttpResponse::InternalServerError().body(msg.clone()),
            Error::SerializationError(msg) => HttpResponse::BadRequest().body(msg.clone()),
            Error::PermissionDenied(msg) => HttpResponse::Forbidden().body(msg.clone()),
            Error::NotFound => HttpResponse::NotFound().body("Session not found".to_string()),
        }
    }
}
