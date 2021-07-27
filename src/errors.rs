use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    HttpError(#[from] reqwest::Error),
    DBError(#[from] quaint::error::Error),
}
