use entity::{
    gen::{chats, players, users},
    prelude::*,
};
use sea_orm::ConnectionTrait;
use teloxide::types::{ChatId, Message};

use crate::{consts::DEFAULT_LOCALE, AppError, AppResult, AppResultExt, ClientError};

pub fn is_mention(arg: &str) -> bool {
    arg.starts_with('@') && !arg.contains(char::is_whitespace)
}

pub fn resolve_mention<'a>(arg: &'a str, cmd: &'static str) -> AppResult<&'a str> {
    if is_mention(arg) {
        Ok(arg.split_at(1).1)
    } else {
        Err(ClientError::NoMention(cmd).into())
    }
}

pub fn get_language_code(msg: &Message) -> &str {
    msg.from()
        .and_then(|user| user.language_code.as_ref())
        .map_or(DEFAULT_LOCALE, |code| code.as_str())
}

pub async fn find_user(db: &impl ConnectionTrait, username: &str) -> AppResult<users::Model> {
    Ok(Users::find_by_username(username)
        .one(db)
        .await?
        .ok_or(ClientError::NoUser(username.to_owned()))?)
}

pub async fn find_msg_context(
    db: &impl ConnectionTrait,
    msg: &Message,
) -> AppResult<(users::Model, chats::Model)> {
    Ok((
        Users::find_by_telegram_id(msg.from().ok_or(AppError::NonExistentSender)?.id)
            .one(db)
            .await?
            .ok_or(AppError::UnknownUser)?,
        Chats::find_by_telegram_id(msg.chat.id)
            .one(db)
            .await?
            .ok_or(AppError::UnknownChat)?,
    ))
}

pub async fn find_msg_player(
    db: &impl ConnectionTrait,
    msg: &Message,
) -> AppResult<(players::Model, users::Model)> {
    Players::find_by_chat_user(
        msg.chat.id,
        msg.from().ok_or(AppError::NonExistentSender)?.id,
    )
    .select_also(Users)
    .one(db)
    .await?
    .ok_or(AppError::UnknownPlayer)
    .map_tuple_with_option(AppError::UnknownUser)
}
pub async fn find_player_by_username(
    db: &impl ConnectionTrait,
    chat_id: ChatId,
    username: &str,
) -> AppResult<(players::Model, users::Model)> {
    Players::find_by_chat_username(chat_id, username)
        .select_also(Users)
        .one(db)
        .await?
        .ok_or(AppError::UnknownPlayer)
        .map_tuple_with_option(AppError::UnknownUser)
}
