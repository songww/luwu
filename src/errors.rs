use std::io::Cursor;

use rocket::response::Responder;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("RequestError {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("Database execute error {0}")]
    DBError(#[from] quaint::error::Error),
    #[error("Database dose not configured.")]
    DBNotAvailable,
    #[error("Processor type is invalid, expect one of `xa`, `tcc`, `saga`, `message`, but `{0}` found.")]
    InvalidProcessorType(String),
}

#[derive(Debug, serde::Serialize)]
struct ErrResponse {
    message: String,
}

impl<'r> Responder<'r, 'static> for Error {
    fn respond_to(self, _request: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        let resp = ErrResponse {
            message: self.to_string(),
        };
        let body = serde_json::to_string(&resp).unwrap();
        rocket::Response::build()
            .header(rocket::http::ContentType::JSON)
            .status(rocket::http::Status::InternalServerError)
            .sized_body(body.len(), Cursor::new(body))
            .ok()
    }
}
