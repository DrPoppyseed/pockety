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

#[derive(Debug)]
pub struct HttpError {
    pub status: reqwest::StatusCode,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
}

impl HttpError {
    pub fn new(status: reqwest::StatusCode) -> Self {
        HttpError {
            status,
            error_code: None,
            error_message: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ApiError {
    MissingAccessToken,
    MissingRequestToken,
}

impl From<reqwest::Error> for Error {
    fn from(_: reqwest::Error) -> Self {
        Error::Http(HttpError::new(reqwest::StatusCode::INTERNAL_SERVER_ERROR))
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
