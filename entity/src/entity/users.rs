use sea_orm::prelude::*;
use teloxide::types::UserId;

use crate::{gen::users::*, prelude::*};

impl Users {
    pub fn find_by_telegram_id(user_id: UserId) -> Select<Self> {
        Self::find().filter(Column::TelegramId.eq(user_id.0.to_string()))
    }

    pub fn find_by_username(username: &str) -> Select<Self> {
        Self::find().filter(Column::Username.eq(username))
    }

    pub fn find_duplicate(user_id: u64, username: &str) -> Select<Self> {
        Self::find_by_username(username).filter(Column::TelegramId.eq(user_id.to_string()).not())
    }
}

impl Model {
    pub fn get_telegram_id(&self) -> UserId {
        UserId(self.telegram_id.parse().unwrap())
    }

    pub fn mention(&self) -> Option<String> {
        self.username.as_ref().map(|name| format!("@{name}"))
    }

    pub fn eq_by_id(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
