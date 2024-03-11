use std::io::BufReader;

use image::{ImageBuffer, Rgb};
use teloxide::types::InputFile;

use crate::AppResult;

pub fn create_pet_gif(width: u32, height: u32, image_buf: Vec<u8>) -> AppResult<InputFile> {
    todo!()
}
