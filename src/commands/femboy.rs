use serenity::{
    framework::standard::{
        macros::{command, group},
        CommandResult,
    },
    model::prelude::*,
    prelude::*,
};

use super::handle_user_error;
use crate::service::ServiceError;
use crate::{service::femboy::*, AppContext};

pub fn handle_femboy_error(e: FemboyError) -> String {
    match e {
        FemboyError::AlreadyRegistered => {
            t!("msg.femboy.register.error.already_registered")
        }
        FemboyError::NoGuildId => {
            t!("msg.femboy.register.error.no_guild")
        }
    }
}

#[group]
#[commands(femboy_register, femboy_leaderboard, femboy)]
pub struct FEMBOY;

#[command]
async fn femboy_register(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let app_ctx = data
        .get::<AppContext>()
        .expect("Expected AppContext")
        .lock()
        .await;

    if let Err(error) = FemboyService::register(&app_ctx, msg.guild_id, msg.author.id).await {
        msg.reply(
            ctx,
            match error {
                ServiceError::DbErr(e) => t!("msg.common.error.db_err", msg = e.to_string()),
                ServiceError::FemboyError(e) => handle_femboy_error(e),
                ServiceError::UserError(e) => handle_user_error(e),
            },
        )
        .await?;
    } else {
        msg.reply(ctx, t!("msg.femboy.register.success")).await?;
    }

    Ok(())
}

#[command]
async fn femboy_leaderboard(_ctx: &Context, _msg: &Message) -> CommandResult {
    todo!()
}

#[command]
async fn femboy(_ctx: &Context, _msg: &Message) -> CommandResult {
    todo!()
}
