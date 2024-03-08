use teloxide::{adaptors::DefaultParseMode, prelude::*};

pub mod command;
pub mod image;

pub type FluffersBot = DefaultParseMode<Bot>;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Url(#[from] url::ParseError),
    #[error(transparent)]
    Request(#[from] reqwest::Error),
    #[error(transparent)]
    TeloxideRequest(#[from] teloxide::RequestError),
    #[error("No image")]
    NoImageFound,
}

pub type AppResult<T> = Result<T, AppError>;
