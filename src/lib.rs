pub mod handlers;
pub mod utils;
use sea_orm::DatabaseConnection;
use teloxide::{adaptors::DefaultParseMode, prelude::*};

pub mod command;
pub mod images;

#[macro_use]
extern crate log;

#[macro_use]
extern crate rust_i18n;
i18n!("locales", fallback = "ru");

pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

pub type DbPool = DatabaseConnection;

pub type FluffersBot = DefaultParseMode<Bot>;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Url(#[from] url::ParseError),
    #[error(transparent)]
    Request(#[from] reqwest::Error),
    #[error(transparent)]
    TeloxideRequest(#[from] teloxide::RequestError),
    #[error(transparent)]
    TeloxideDownload(#[from] teloxide::DownloadError),
    #[error(transparent)]
    Database(#[from] sea_orm::DbErr),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Image(#[from] image::ImageError),
    #[error(transparent)]
    Gif(#[from] gif::EncodingError),
    #[error("No image")]
    NoImageFound,
}

pub type AppResult<T> = Result<T, AppError>;
