pub mod utils;
pub mod handlers;
use sea_orm::DatabaseConnection;
use teloxide::{adaptors::DefaultParseMode, prelude::*};

pub mod command;
pub mod image;

#[macro_use]
extern crate log;

#[macro_use]
extern crate rust_i18n;
i18n!("locales", fallback = "ru");

// pub type DbConnectionType = diesel::PgConnection;
// pub type DbPool = Pool<ConnectionManager<DbConnectionType>>;
// pub type DbConnection = PooledConnection<ConnectionManager<DbConnectionType>>;
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
    Database(#[from] sea_orm::DbErr),
    #[error("No image")]
    NoImageFound,
}

pub type AppResult<T> = Result<T, AppError>;
