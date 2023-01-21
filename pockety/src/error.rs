#[derive(Debug)]
pub enum Error {
    Http(HttpError),
    Api(ApiError),
    Json(String),
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
    fn from(error: reqwest::Error) -> Self {
        Error::Http(HttpError::new(reqwest::StatusCode::INTERNAL_SERVER_ERROR))
    }
}

impl From<reqwest::header::ToStrError> for Error {
    fn from(error: reqwest::header::ToStrError) -> Self {
        Error::Json(error.to_string())
    }
}
