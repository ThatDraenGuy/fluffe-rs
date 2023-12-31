use serenity::{
    framework::standard::{
        macros::{command, group},
        CommandResult,
    },
    model::prelude::*,
    prelude::*,
    utils::MessageBuilder,
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
            t!("msg.femboy.common.error.no_guild")
        }
        FemboyError::NoFemboyFound => {
            t!("msg.femboy.common.error.no_femboy_found")
        }
        FemboyError::NoUserFound => {
            t!("msg.femboy.common.error.no_user_found")
        }
    }
}

#[group]
#[commands(femboy_register, femboy_leaderboard, femboy, balance)]
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
async fn femboy_leaderboard(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let app_ctx = data
        .get::<AppContext>()
        .expect("Expected AppContext")
        .lock()
        .await;

    match FemboyService::get_femboy_leaderboard(&app_ctx, msg.guild_id).await {
        Err(error) => {
            msg.reply(
                ctx,
                match error {
                    ServiceError::DbErr(e) => t!("msg.common.error.db_err", msg = e.to_string()),
                    ServiceError::FemboyError(e) => handle_femboy_error(e),
                    ServiceError::UserError(e) => handle_user_error(e),
                },
            )
            .await?;
        }

        Ok(leaderboard) => {
            let guild = msg.guild(&ctx.cache).unwrap();

            let mut response = MessageBuilder::new();

            response
                .push(t!("msg.femboy.leaderboard.success"))
                .push("\n");

            for (i, (femboy, user)) in leaderboard.iter().enumerate() {
                let member = guild
                    .member(&ctx.http, &UserId::from(user.discord_id.parse::<u64>()?))
                    .await?;

                response
                    .push(t!(
                        "msg.femboy.leaderboard.line",
                        place = i,
                        name = member.display_name(),
                        wins_num = femboy.wins_num
                    ))
                    .push("\n");
            }

            msg.reply(ctx, response.build().trim()).await?;
        }
    }

    Ok(())
}

#[command]
async fn femboy(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let app_ctx = data
        .get::<AppContext>()
        .expect("Expected AppContext")
        .lock()
        .await;

    match FemboyService::choose(&app_ctx, msg.guild_id).await {
        Err(error) => {
            msg.reply(
                ctx,
                match error {
                    ServiceError::DbErr(e) => t!("msg.common.error.db_err", msg = e.to_string()),
                    ServiceError::FemboyError(e) => handle_femboy_error(e),
                    ServiceError::UserError(e) => handle_user_error(e),
                },
            )
            .await?;
        }
        Ok((winnings, femboy, user)) => {
            let response = MessageBuilder::new()
                .push(t!("msg.femboy.choose.success.begin"))
                .mention(&UserId::from(user.discord_id.parse::<u64>()?))
                .push(t!(
                    "msg.femboy.choose.success.end",
                    winnings = winnings,
                    wins_num = femboy.wins_num
                ))
                .build();

            msg.channel_id.say(&ctx.http, response).await?;
        }
    }
    Ok(())
}

#[command]
async fn balance(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let app_ctx = data
        .get::<AppContext>()
        .expect("Expected AppContext")
        .lock()
        .await;

    match FemboyService::find(&app_ctx, msg.guild_id, msg.author.id).await {
        Err(error) => {
            msg.reply(
                ctx,
                match error {
                    ServiceError::DbErr(e) => t!("msg.common.error.db_err", msg = e.to_string()),
                    ServiceError::FemboyError(e) => handle_femboy_error(e),
                    ServiceError::UserError(e) => handle_user_error(e),
                },
            )
            .await?;
        }
        Ok((femboy, _)) => {
            let response = MessageBuilder::new()
                .mention(&msg.author.id)
                .push(t!("msg.femboy.balance.success", balance = femboy.balance))
                .build();

            msg.channel_id.say(&ctx.http, response).await?;
        }
    }
    Ok(())
}
