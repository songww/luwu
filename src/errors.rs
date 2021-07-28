use thiserror::Error;

#[derive(Debug, Display, Error)]
pub enum Error {
    HttpError(#[from] reqwest::Error),
    DBError(#[from] quaint::error::Error),
}
