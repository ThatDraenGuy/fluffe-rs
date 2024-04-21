use entity::prelude::*;

use sea_orm::ActiveModelTrait;
use sea_orm::ConnectionTrait;
use sea_orm::IntoActiveModel;
use sea_orm::TransactionTrait;
use sea_orm::{EntityTrait, Set};
use std::sync::Arc;
use teloxide::types::ChatId;
use teloxide::types::Update;
use teloxide::types::User;

use crate::{AppResult, DbPool};

pub async fn unhandled_update_logging_handler(upd: Arc<Update>) {
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

pub async fn update_logging_handler(upd: Update) -> bool {
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
    if let Err(e) = handle_username_storage(upd, db).await {
        warn!("Error when updating username data: {e}");
    }

    true
}

async fn handle_username_storage(upd: Update, db: DbPool) -> AppResult<()> {
    let Some(user) = upd.user() else {
        return Ok(());
    };

    let t = db.begin().await?;
    let user_id = update_username(user, &t).await?;
    t.commit().await?;

    if let Some(chat) = upd.chat() {
        let t = db.begin().await?;
        let chat_id = update_chat(chat.id, &t).await?;
        t.commit().await?;

        let t = db.begin().await?;
        update_player(chat_id, user_id, &t).await?;
        t.commit().await?;
    }
    Ok(())
}

async fn update_username(msg_user: &User, conn: &impl ConnectionTrait) -> AppResult<i64> {
    let user_id = msg_user.id;

    let maybe_user_model = Users::find_by_telegram_id(user_id).one(conn).await?;

    let Some(msg_username) = &msg_user.username else {
        // if User has no username

        if let Some(user_model) = maybe_user_model {
            // if we found this user than everything is OK
            return Ok(user_model.id);
        };

        // otherwise we got message from a new user
        let new_user = ActiveUsers {
            telegram_id: Set(user_id.to_string()),
            username: Set(None),
            ..Default::default()
        };

        let res = Users::insert(new_user).exec(conn).await?;

        info!("Registered new user [ID:{user_id}] with no username");

        return Ok(res.last_insert_id);
    };

    // if User has a username - check that it matches the one in db
    if let Some(user_model) = &maybe_user_model {
        if user_model
            .username
            .as_ref()
            .is_some_and(|db_username| db_username == msg_username)
        {
            // if usernames are the same - nothing changed, i.e. we don't need to do anything
            return Ok(user_model.id);
        }
    }

    // if usernames don't match check for that username in db
    if let Some(other_user) = Users::find_duplicate(user_id.0, msg_username)
        .one(conn)
        .await?
    {
        // There is another user who had that username previously, then changed it and had his original username taken by the new user
        // basically, we need to clear it

        let mut other_user = other_user.into_active_model();
        other_user.username = Set(None);
        other_user.save(conn).await?;
        info!("Cleared username [{msg_username}] from user with ID [{user_id}]");
    }

    // now there is edfinetely no collisions in db, and we have either a new guy, or a guy with changed username
    match maybe_user_model {
        Some(user) => {
            // Found this user_id in database, i.e. we know this guy

            // Update record with new username
            let mut user = user.into_active_model();
            user.username = Set(Some(msg_username.to_owned()));
            let res = user.update(conn).await?;

            info!("Updated our user [{msg_username}] with ID [{user_id}]");
            Ok(res.id)
        }
        None => {
            // No user_id in database - this is a new guy.
            // Create record for new user

            let new_user = ActiveUsers {
                telegram_id: Set(user_id.to_string()),
                username: Set(Some(msg_username.to_owned())),
                ..Default::default()
            };

            let res = Users::insert(new_user).exec(conn).await?;

            info!("Registered new user [{msg_username}] with ID [{user_id}]");
            Ok(res.last_insert_id)
        }
    }
}

async fn update_chat(chat_telegram_id: ChatId, conn: &impl ConnectionTrait) -> AppResult<i64> {
    let maybe_chat = Chats::find_by_telegram_id(chat_telegram_id)
        .one(conn)
        .await?;

    if let Some(chat) = maybe_chat {
        Ok(chat.id)
    } else {
        let new_chat = ActiveChats {
            telegram_id: Set(chat_telegram_id.0),
            ..Default::default()
        };

        let res = Chats::insert(new_chat).exec(conn).await?;

        info!("Registered new chat with ID [{chat_telegram_id}]");
        Ok(res.last_insert_id)
    }
}

async fn update_player(chat_id: i64, user_id: i64, conn: &impl ConnectionTrait) -> AppResult<()> {
    let maybe_player = Players::find_by_logical_key(chat_id, user_id)
        .one(conn)
        .await?;

    if maybe_player.is_none() {
        let new_player = ActivePlayers {
            user_id: Set(user_id),
            chat_id: Set(chat_id),
            ..Default::default()
        };
        Players::insert(new_player).exec(conn).await?;
        info!("Registered new player for chat [{chat_id}] & user [{user_id}]");
    }
    Ok(())
}
