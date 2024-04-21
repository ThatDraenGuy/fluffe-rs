use sea_orm::DatabaseConnection;
use teloxide::{adaptors::DefaultParseMode, prelude::*};

pub mod command;
pub mod consts;
pub mod handlers;
pub mod images;
pub mod utils;

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
    #[error("Unknown chat")]
    UnknownChat,
    #[error("Unknown user")]
    UnknownUser,
    #[error("Unknown player")]
    UnknownPlayer,
    #[error("No sender")]
    NonExistentSender,
    #[error("No image")]
    NoImageFound,
    #[error(transparent)]
    ClientError(#[from] ClientError),
}

#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    #[error("No mention whe using command {0}")]
    NoMention(&'static str),
    #[error("No user for username {0} found")]
    NoUser(String),
}

pub type AppResult<T> = Result<T, AppError>;

pub trait AppResultExt<T, R> {
    fn map_tuple_with_option(self, err: AppError) -> AppResult<(T, R)>;
}
impl<T, R> AppResultExt<T, R> for AppResult<(T, Option<R>)> {
    fn map_tuple_with_option(self, err: AppError) -> AppResult<(T, R)> {
        self.and_then(|tup| tup.1.map(|val| (tup.0, val)).ok_or(err))
    }
}
