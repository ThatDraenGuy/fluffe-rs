use entity::{
    gen::{chats, players, users},
    prelude::*,
};
use sea_orm::ConnectionTrait;
use teloxide::types::{ChatId, Message};

use crate::{
    consts::{DEFAULT_LOCALE, DEFAULT_TOP_LIMIT},
    AppError, AppResult, AppResultExt, ClientError, VecTupleExt,
};

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

pub async fn find_user(conn: &impl ConnectionTrait, username: &str) -> AppResult<users::Model> {
    Ok(Users::find_by_username(username)
        .one(conn)
        .await?
        .ok_or(ClientError::NoUser(username.to_owned()))?)
}

pub async fn find_msg_context(
    conn: &impl ConnectionTrait,
    msg: &Message,
) -> AppResult<(users::Model, chats::Model)> {
    Ok((
        Users::find_by_telegram_id(msg.from().ok_or(AppError::NonExistentSender)?.id)
            .one(conn)
            .await?
            .ok_or(AppError::UnknownUser)?,
        Chats::find_by_telegram_id(msg.chat.id)
            .one(conn)
            .await?
            .ok_or(AppError::UnknownChat)?,
    ))
}

pub async fn find_msg_player(
    conn: &impl ConnectionTrait,
    msg: &Message,
) -> AppResult<(players::Model, users::Model)> {
    Players::find_by_chat_user(
        msg.chat.id,
        msg.from().ok_or(AppError::NonExistentSender)?.id,
    )
    .select_also(Users)
    .one(conn)
    .await?
    .ok_or(AppError::UnknownPlayer)
    .map_tuple_with_option(AppError::UnknownUser)
}
pub async fn find_player_by_username(
    conn: &impl ConnectionTrait,
    chat_id: ChatId,
    username: &str,
) -> AppResult<(players::Model, users::Model)> {
    Players::find_by_chat_username(chat_id, username)
        .select_also(Users)
        .one(conn)
        .await?
        .ok_or(AppError::UnknownPlayer)
        .map_tuple_with_option(AppError::UnknownUser)
}

pub async fn find_top_pets_given(
    conn: &impl ConnectionTrait,
    chat_id: ChatId,
) -> AppResult<Vec<(players::Model, users::Model)>> {
    Players::find_top_in_chat(chat_id, players::Column::PetsGiven, DEFAULT_TOP_LIMIT)
        .inner_join(Users)
        .select_also(Users)
        .all(conn)
        .await?
        .map_tuple_with_options(AppError::UnknownUser)
}
pub async fn find_top_pets_received(
    conn: &impl ConnectionTrait,
    chat_id: ChatId,
) -> AppResult<Vec<(players::Model, users::Model)>> {
    Players::find_top_in_chat(chat_id, players::Column::PetsReceived, DEFAULT_TOP_LIMIT)
        .inner_join(Users)
        .select_also(Users)
        .all(conn)
        .await?
        .map_tuple_with_options(AppError::UnknownUser)
}

pub fn format_as_top_list<T, F>(items: &[(T, users::Model)], top_value_getter: F) -> String
where
    F: Fn(&T) -> String,
{
    items
        .iter()
        .enumerate()
        .fold(String::new(), |acc, (index, (player, user))| {
            format!(
                "{acc}{index}. {mention} - {value}\n",
                index = index + 1,
                mention = user.username.as_deref().unwrap_or("Someone"),
                value = top_value_getter(player)
            )
        })
}
