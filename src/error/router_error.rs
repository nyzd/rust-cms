use actix_web::{
    error::ResponseError,
    http::{header::ContentType, StatusCode},
    HttpResponse,
};
use std::fmt::Display;

#[derive(Debug)]
pub enum RouterError {
    /// String is a message associated with the error
    /// response
    Auth(String),
}

impl Display for RouterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auth(message) => write!(f, "{}", message),
        }
    }
}

impl ResponseError for RouterError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::plaintext())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match self {
            Self::Auth(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
