use std::sync::Arc;

use include_bytes_plus::include_bytes;
use teloxide::{
    net::Download,
    payloads::{SendAnimationSetters, SendMessageSetters, SendStickerSetters},
    requests::{Requester, ResponseResult},
    types::{InputFile, Message, UserId},
    utils::command::BotCommands,
};

use crate::{
    built_info,
    images::{pet_gif_creator::create_pet_gif, ImageRepository, ImageRepositoryTrait},
    utils::{get_language_code, is_mention, DEFAULT_MENTION},
    AppResult, DbPool, FluffersBot,
};

use entity::prelude::*;

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "snake_case")]
pub enum AppCommands {
    // GetFurry, // temporarily disabled
    Pet(String),
    Shipu,
    About,
}

pub async fn handle_command(
    // image_repository: Arc<ImageRepository>,
    bot: FluffersBot,
    msg: Message,
    cmd: AppCommands,
    db: DbPool,
) -> ResponseResult<()> {
    let user_id = msg.from().map_or(0, |u| u.id.0);
    let chat_id = msg.chat.id;

    let result = match cmd.clone() {
        // AppCommands::GetFurry => get_furry(image_repository, bot, &msg).await,
        AppCommands::Pet(arg) => pet(&db, bot, &msg, &arg).await,
        AppCommands::Shipu => shipu(bot, &msg).await,
        AppCommands::About => about(bot, &msg).await,
    };

    match result {
        Ok(_) => log::info!("Handled command /{cmd:?}: from user [{user_id}] in chat [{chat_id}]",),
        Err(e) => log::error!(
            "Error {e:?}: on command /{cmd:?}: from user [{user_id}] in chat [{chat_id}]",
        ),
    }

    Ok(())
}

#[allow(unused)]
async fn get_furry(
    image_repository: Arc<ImageRepository>,
    bot: FluffersBot,
    msg: &Message,
) -> AppResult<()> {
    let image = image_repository.get_random_image().await?;
    bot.send_photo(msg.chat.id, image).await?;
    Ok(())
}

async fn pet(db: &DbPool, bot: FluffersBot, msg: &Message, mention: &str) -> AppResult<()> {
    if !is_mention(mention) {
        bot.send_message(
            msg.chat.id,
            t!(
                "msg.common.error.mention_argument",
                command = "pet",
                locale = get_language_code(msg),
                mention = DEFAULT_MENTION,
            ),
        )
        .reply_to_message_id(msg.id)
        .await?;
        return Ok(());
    }
    let username = mention.split_at(1).1;

    let Some(user) = Users::find_by_username(username).one(db).await? else {
        bot.send_message(
            msg.chat.id,
            t!(
                "msg.common.error.unknown_username",
                locale = get_language_code(msg),
                mention = mention,
            ),
        )
        .reply_to_message_id(msg.id)
        .await?;
        return Ok(());
    };
    let user_id = UserId(user.get_telegram_id());

    let user_photos = bot.get_user_profile_photos(user_id).await?;
    let Some(photo) = user_photos.photos.first().and_then(|photo| photo.first()) else {
        bot.send_message(
            msg.chat.id,
            t!(
                "msg.pet.error.no_photo",
                locale = get_language_code(msg),
                mention = mention,
            ),
        )
        .reply_to_message_id(msg.id)
        .await?;
        return Ok(());
    };

    let file = bot.get_file(&photo.file.id).await?;

    let mut avatar = Vec::with_capacity(file.size as usize);
    bot.download_file(&file.path, &mut avatar).await?;

    let gif = create_pet_gif(avatar, username)?;

    let gif_msg = bot
        .send_animation(msg.chat.id, gif)
        .reply_to_message_id(msg.id)
        .await?;
    bot.send_message(
        msg.chat.id,
        t!(
            "msg.pet.success",
            locale = get_language_code(msg),
            target = mention
        ),
    )
    .reply_to_message_id(gif_msg.id)
    .await?;

    Ok(())
}

const SHIPU_STICKER: [u8; 36758] = include_bytes!("resources/shipu.webp");

async fn shipu(bot: FluffersBot, msg: &Message) -> AppResult<()> {
    bot.send_sticker(msg.chat.id, InputFile::memory(&SHIPU_STICKER as &[u8]))
        .reply_to_message_id(msg.id.0)
        .await?;

    Ok(())
}

async fn about(bot: FluffersBot, msg: &Message) -> AppResult<()> {
    let last_update = built::util::strptime(built_info::BUILT_TIME_UTC).format("%d-%m-%Y %H:%M:%S");

    bot.send_message(
        msg.chat.id,
        t!(
            "msg.about.info",
            locale = get_language_code(msg),
            version = built_info::PKG_VERSION,
            authors = built_info::PKG_AUTHORS,
            last_update = last_update
        ),
    )
    .reply_to_message_id(msg.id)
    .await?;

    Ok(())
}
