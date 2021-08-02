use std::io::Cursor;

use rocket::response::Responder;
use thiserror::Error;

use crate::models::transaction::Gid;

#[derive(Debug, Error)]
pub enum Error {
    #[error("RequestError {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("Database execute error {0}")]
    DBError(#[from] quaint::error::Error),
    #[error("Database dose not configured.")]
    DBNotAvailable,
    #[error(
        "Processor type is invalid, expect one of `xa`, `tcc`, `saga`, `message`, but `{0}` found."
    )]
    InvalidProcessorType(String),
    #[error("Unexpected transaction type {0} with state {1}, {0}")]
    UnexpectedType(String, String, String),
    #[error("Can not submit transaction({0}) from {1}.")]
    CannotSubmitTransaction(Gid, String),
}

#[derive(Debug, serde::Serialize)]
struct ErrResponse {
    message: String,
    code: u16,
}

pub struct ErrorResponse {
    code: u16,
    err: Error,
}

impl ErrorResponse {
    pub fn new(err: Error, code: u16) -> ErrorResponse {
        ErrorResponse { err, code }
    }
}

impl From<Error> for ErrorResponse {
    fn from(err: Error) -> Self {
        ErrorResponse { err, code: 5999 }
    }
}

impl<'r> Responder<'r, 'static> for ErrorResponse {
    fn respond_to(self, _request: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        let resp = ErrResponse {
            message: self.err.to_string(),
            code: self.code,
        };
        let body = serde_json::to_string(&resp).unwrap();
        rocket::Response::build()
            .header(rocket::http::ContentType::JSON)
            .status(rocket::http::Status::InternalServerError)
            .sized_body(body.len(), Cursor::new(body))
            .ok()
    }
}
