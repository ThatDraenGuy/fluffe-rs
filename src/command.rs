use std::sync::Arc;

use image::ImageBuffer;
use teloxide::{
    net::Download,
    payloads::SendMessageSetters,
    requests::{Requester, ResponseResult},
    types::{Message, UserId},
    utils::command::BotCommands,
};
use tokio::fs;

use crate::{
    image::{ImageRepository, ImageRepositoryTrait},
    utils::{get_language_code, is_mention, DEFAULT_MENTION},
    AppResult, DbPool, FluffersBot,
};

use entity::prelude::*;

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "snake_case")]
pub enum AppCommands {
    GetFurry,
    Pet(String),
}

pub async fn handle_command(
    image_repository: Arc<ImageRepository>,
    bot: FluffersBot,
    msg: Message,
    cmd: AppCommands,
    db: DbPool,
) -> ResponseResult<()> {
    let user_id = msg.from().map_or(0, |u| u.id.0);
    let chat_id = msg.chat.id;

    let result = match cmd.clone() {
        AppCommands::GetFurry => get_furry(image_repository, bot, &msg).await,
        AppCommands::Pet(arg) => pet(&db, bot, &msg, &arg).await,
    };

    match result {
        Ok(_) => log::info!("Handled command /{cmd:?}: from user [{user_id}] in chat [{chat_id}]",),
        Err(e) => log::error!(
            "Error {e:?}: on command /{cmd:?}: from user [{user_id}] in chat [{chat_id}]",
        ),
    }

    Ok(())
}

async fn get_furry(
    image_repository: Arc<ImageRepository>,
    bot: FluffersBot,
    msg: &Message,
) -> AppResult<()> {
    let image = image_repository.get_random_image().await?;
    bot.send_photo(msg.chat.id, image).await?;
    Ok(())
}

async fn pet(db: &DbPool, bot: FluffersBot, msg: &Message, arg: &str) -> AppResult<()> {
    if !is_mention(arg) {
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

    let Some(user) = Users::find_by_username(arg.split_at(1).1).one(db).await? else {
        bot.send_message(
            msg.chat.id,
            t!(
                "msg.common.error.unknown_username",
                locale = get_language_code(msg),
                mention = arg,
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
                mention = arg,
            ),
        )
        .reply_to_message_id(msg.id)
        .await?;
        return Ok(());
    };

    let file = bot.get_file(&photo.file.id).await?;

    let mut image_buf = Vec::with_capacity(file.size as usize);

    bot.download_file(&file.path, &mut image_buf).await?;

    Ok(())
}
