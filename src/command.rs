use std::sync::Arc;

use teloxide::{
    payloads::SendMessageSetters,
    requests::{Requester, ResponseResult},
    types::{Message, UserId},
    utils::command::BotCommands,
};

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
                mention = DEFAULT_MENTION,
                locale = get_language_code(msg)
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
                mention = arg,
                locale = get_language_code(msg)
            ),
        )
        .reply_to_message_id(msg.id)
        .await?;
        return Ok(());
    };
    let _user_id = UserId(user.get_telegram_id());

    Ok(())
}
