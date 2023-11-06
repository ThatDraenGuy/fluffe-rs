use sea_orm::DbErr;
use thiserror::Error;

use self::{femboy::FemboyError, user::UserError};

pub mod femboy;
pub mod server;
pub mod user;

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error(transparent)]
    DbErr(#[from] DbErr),
    #[error(transparent)]
    FemboyError(#[from] FemboyError),
    #[error(transparent)]
    UserError(#[from] UserError),
}
