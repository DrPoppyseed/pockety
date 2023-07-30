use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    Http(HttpError),
    Api(ApiError),
    Json(String),
    Parse(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

    pub fn error_code(self, error_code: Option<String>) -> Self {
        Self { error_code, ..self }
    }

    pub fn error_message(self, error_message: Option<String>) -> Self {
        Self {
            error_message,
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
        Error::Http(HttpError::new().error_message(Some(error.to_string())))
    }
}

impl From<reqwest::header::ToStrError> for Error {
    fn from(error: reqwest::header::ToStrError) -> Self {
        Error::Json(error.to_string())
    }
}

impl From<url::ParseError> for Error {
    fn from(error: url::ParseError) -> Self {
        Error::Parse(error.to_string())
    }
}
