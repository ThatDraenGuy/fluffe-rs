use std::io::Cursor;

use gif::{DisposalMethod, Frame};
use image::io::Reader as ImageReader;
use image::{
    imageops::{overlay, FilterType},
    DynamicImage, ImageFormat,
};
use teloxide::types::InputFile;

use crate::AppResult;

const HAND_SPRITE_PATH: &str = "resources/hand_sprite.png";

const FRAME_SIZE: u32 = 112;

const AVATAR_FRAME_PARAMS: [FrameParams; 5] = [
    FrameParams {
        x: 14,
        y: 14,
        w: 98,
        h: 98,
    },
    FrameParams {
        x: 10,
        y: 26,
        w: 102,
        h: 86,
    },
    FrameParams {
        x: 2,
        y: 32,
        w: 110,
        h: 80,
    },
    FrameParams {
        x: 6,
        y: 26,
        w: 102,
        h: 86,
    },
    FrameParams {
        x: 10,
        y: 14,
        w: 98,
        h: 98,
    },
];

struct FrameParams {
    x: i64,
    y: i64,
    w: u32,
    h: u32,
}

pub fn create_pet_gif(avatar_image: Vec<u8>, username: &str) -> AppResult<InputFile> {
    let avatar = ImageReader::with_format(Cursor::new(avatar_image), ImageFormat::Jpeg).decode()?;
    let hand_frames = ImageReader::open(HAND_SPRITE_PATH)?
        .with_guessed_format()?
        .decode()?;

    let mut out = Vec::new();
    let mut encoder = gif::Encoder::new(&mut out, FRAME_SIZE as u16, FRAME_SIZE as u16, &[])?;
    encoder.set_repeat(gif::Repeat::Infinite)?;

    for frame_index in 0..5u32 {
        let frame_params = &AVATAR_FRAME_PARAMS[frame_index as usize];
        let avatar = avatar.resize_exact(frame_params.w, frame_params.h, FilterType::Triangle);
        let hand = hand_frames.crop_imm(FRAME_SIZE * frame_index, 0, FRAME_SIZE, FRAME_SIZE);

        let mut canvas = DynamicImage::new_rgba8(FRAME_SIZE, FRAME_SIZE);
        overlay(&mut canvas, &avatar, frame_params.x, frame_params.y);
        overlay(&mut canvas, &hand, 0, 0);

        let mut frame = Frame::from_rgba(
            FRAME_SIZE as u16,
            FRAME_SIZE as u16,
            &mut canvas.into_bytes(),
        );
        frame.dispose = DisposalMethod::Background;

        encoder.write_frame(&frame)?;
    }
    drop(encoder);

    Ok(InputFile::memory(out).file_name(format!("pet-{username}.gif")))
}
