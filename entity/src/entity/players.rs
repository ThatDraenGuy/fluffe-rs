use sea_orm::{prelude::*, Order, QueryOrder, QuerySelect};
use teloxide::types::{ChatId, UserId};

use crate::{
    gen::{chats, players::*, users},
    prelude::*,
};

impl Players {
    pub fn find_by_chat_id(chat_id: ChatId) -> Select<Players> {
        Self::find()
            .inner_join(Chats)
            .filter(chats::Column::TelegramId.eq(chat_id.0))
    }

    pub fn find_by_logical_key(chat_id: i64, user_id: i64) -> Select<Players> {
        Self::find()
            .filter(Column::ChatId.eq(chat_id))
            .filter(Column::UserId.eq(user_id))
    }

    pub fn find_by_chat_user(chat_id: ChatId, user_id: UserId) -> Select<Players> {
        Self::find_by_chat_id(chat_id)
            .inner_join(Users)
            .filter(users::Column::TelegramId.eq(user_id.0.to_string()))
    }

    pub fn find_by_chat_username(chat_id: ChatId, username: &str) -> Select<Players> {
        Self::find_by_chat_id(chat_id)
            .inner_join(Users)
            .filter(users::Column::Username.eq(username))
    }

    pub fn find_top_in_chat(chat_id: ChatId, col: Column, limit: u64) -> Select<Players> {
        Self::find_by_chat_id(chat_id)
            .order_by(col, Order::Desc)
            .limit(limit)
    }
}
