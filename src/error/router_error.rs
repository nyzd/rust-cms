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

    /// 500 Error
    InternalError,

    /// 404 NotFound
    /// with message to the client
    NotFound(String),

    /// 403 Gone
    /// token or emailverification can expire
    Expired(String),
}

impl Display for RouterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auth(message) => write!(f, "{}", message),
            Self::NotFound(message) => write!(f, "{}", message),
            Self::Expired(message) => write!(f, "{}", message),
            Self::InternalError => write!(f, "InternalError")
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
            Self::Auth(_) => StatusCode::UNAUTHORIZED,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Expired(_) => StatusCode::GONE,
            Self::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
