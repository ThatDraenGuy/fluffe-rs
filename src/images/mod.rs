use std::future::Future;

use enum_dispatch::enum_dispatch;
use reactor::ReactorRepository;
use teloxide::types::InputFile;

use crate::AppResult;

pub mod pet_gif_creator;
pub mod reactor;

#[enum_dispatch]
pub trait ImageRepositoryTrait {
    fn get_random_image(&self) -> impl Future<Output = AppResult<InputFile>> + Send;
}

#[enum_dispatch(ImageRepositoryTrait)]
pub enum ImageRepository {
    ReactorRepository,
}
