use sea_orm::prelude::*;

use crate::{
    gen::users::{Column, Model},
    prelude::*,
};

impl Users {
    pub fn find_by_user_id(user_id: u64) -> Select<Self> {
        Self::find().filter(Column::TelegramId.eq(user_id.to_string()))
    }

    pub fn find_by_username(username: &str) -> Select<Self> {
        Self::find().filter(Column::Username.eq(username))
    }

    pub fn find_duplicate(user_id: u64, username: &str) -> Select<Self> {
        Self::find_by_username(username).filter(Column::TelegramId.eq(user_id.to_string()).not())
    }
}

impl Model {
    pub fn get_telegram_id(&self) -> u64 {
        self.telegram_id.parse().unwrap()
    }
}
