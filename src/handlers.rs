use entity::prelude::*;

use sea_orm::ActiveModelTrait;
use sea_orm::IntoActiveModel;
use sea_orm::{EntityTrait, Set};
use std::sync::Arc;
use teloxide::types::Update;
use teloxide::types::User;

use crate::{AppResult, DbPool};

pub async fn handle_unhandled_update_logging(upd: Arc<Update>) {
    let update_id = upd.id;
    if let Some(user) = upd.user() {
        let user_id = user.id;
        if let Some(chat) = upd.chat() {
            let chat_id = chat.id;
            warn!("Unhandled update [{update_id}]: user: [{user_id}] chat: [{chat_id}]");
        } else {
            warn!("Unhandled update [{update_id}]: user: [{user_id}] ");
        };
    } else if let Some(chat) = upd.chat() {
        let chat_id = chat.id;
        warn!("Unhandled update [{update_id}]: chat: [{chat_id}]");
    } else {
        warn!("Unhandled update [{update_id}]: kind: {:?}", upd.kind);
    }
}

pub async fn handle_update_logging(upd: Update) -> bool {
    let update_id = upd.id;

    let Some(user) = upd.user() else {
        if let Some(chat) = upd.chat() {
            let chat_id = chat.id;
            info!("Received update [{update_id}]: chat: [{chat_id}]");
        } else {
            info!("Received update [{update_id}]: kind: {:?}", upd.kind);
        }
        return true;
    };

    let user_id = user.id;
    if let Some(chat) = upd.chat() {
        let chat_id = chat.id;
        info!("Received update [{update_id}]: user: [{user_id}] chat: [{chat_id}]");
    } else {
        info!("Received update [{update_id}]: user: [{user_id}] ");
    };

    true
}

pub async fn username_storage_handler(upd: Update, db: DbPool) -> bool {
    let Some(user) = upd.user() else {
        return true;
    };

    if let Err(e) = handle_usernames(user, &db).await {
        warn!("Error when updating username data: {e}");
    }

    true
}

async fn handle_usernames(msg_user: &User, db: &DbPool) -> AppResult<()> {
    let user_id = msg_user.id;

    let maybe_user = Users::find_by_user_id(user_id.0).one(db).await?;

    let Some(username) = &msg_user.username else {
        // if User has no username

        if maybe_user.is_some() {
            // if we found this user than everything is OK
            return Ok(());
        };

        // otherwise we got message from a new user
        let new_user = ActiveUsers {
            telegram_id: Set(user_id.to_string()),
            username: Set(None),
            ..Default::default()
        };

        Users::insert(new_user).exec(db).await?;

        info!("Registered new user [ID:{user_id}] with no username");

        return Ok(());
    };

    if maybe_user
        .as_ref()
        .is_some_and(|user| user.username.as_ref().is_some_and(|n| n == username))
    {
        // If usernames are the same - nothing changed, i.e. we don't need to do anything

        info!("Received update from known user [{username}]");
        return Ok(());
    }

    if let Some(other_user) = Users::find_duplicate(user_id.0, username).one(db).await? {
        // There is another user who had that username previously, we need to clear it

        let mut other_user = other_user.into_active_model();
        other_user.username = Set(None);
        other_user.save(db).await?;
        info!("Cleared username [{username}] from user with ID [{user_id}]");
    }

    match maybe_user {
        Some(user) => {
            // Found this user_id in database, i.e. we know this guy

            // Update record with new username
            let mut user = user.into_active_model();
            user.username = Set(Some(username.to_owned()));
            user.update(db).await?;

            info!("Updated our user [{username}] with ID [{user_id}]");
        }
        None => {
            // No user_id in database - this is a new guy.
            // Create record for new user

            let new_user = ActiveUsers {
                telegram_id: Set(user_id.to_string()),
                username: Set(Some(username.to_owned())),
                ..Default::default()
            };

            Users::insert(new_user).exec(db).await?;

            info!("Registered new user [{username}] with ID [{user_id}]");
        }
    }

    Ok(())
}
