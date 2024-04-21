use sea_orm::prelude::*;
use teloxide::types::ChatId;

use crate::gen::chats::*;
use crate::prelude::*;

impl Chats {
    pub fn find_by_telegram_id(chat_id: ChatId) -> Select<Chats> {
        Self::find().filter(Column::TelegramId.eq(chat_id.0))
    }
}
