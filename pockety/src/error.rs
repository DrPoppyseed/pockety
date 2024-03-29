use std::fmt::{Display, Formatter, Result};

use crate::RateLimits;

#[derive(Debug)]
pub enum Error {
    Http(HttpError),
    Api(ApiError),
    Json(String),
    Parse(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Error::Http(error) => write!(f, "Http error: {error:?}"),
            Error::Api(error) => write!(f, "Api error: {error:?}"),
            Error::Json(error) => write!(f, "Json error: {error:?}"),
            Error::Parse(error) => write!(f, "Parse error: {error:?}"),
        }
    }
}

#[derive(Debug, Default)]
pub struct HttpError {
    pub status: reqwest::StatusCode,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub rate_limits: RateLimits,
}

impl HttpError {
    pub fn new() -> Self {
        HttpError {
            status: reqwest::StatusCode::INTERNAL_SERVER_ERROR,
            ..Default::default()
        }
    }

    pub fn status_code(self, status: reqwest::StatusCode) -> Self {
        Self { status, ..self }
    }

    pub fn error_code<T: ToString>(self, error_code: T) -> Self {
        Self {
            error_code: Some(error_code.to_string()),
            ..self
        }
    }

    pub fn error_message<T: ToString>(self, error_message: T) -> Self {
        Self {
            error_message: Some(error_message.to_string()),
            ..self
        }
    }

    pub fn rate_limits(self, rate_limits: RateLimits) -> Self {
        Self {
            rate_limits,
            ..self
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ApiError {
    MissingAccessToken,
    MissingRequestToken,
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Error::Http(HttpError::new().error_message(error))
    }
}

impl From<reqwest::header::ToStrError> for Error {
    fn from(error: reqwest::header::ToStrError) -> Self {
        Error::Json(error.to_string())
    }
}
