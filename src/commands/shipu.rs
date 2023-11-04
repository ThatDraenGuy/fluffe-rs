use rust_i18n::t;
use serenity::{
    framework::standard::{
        macros::{command, group},
        CommandResult,
    },
    model::prelude::*,
    prelude::*,
};

#[group]
#[commands(oleg_shipulin)]
pub struct SHIPU;

const SHIPU_ID: UserId = UserId(373748719012806656);

#[command]
async fn oleg_shipulin(ctx: &Context, msg: &Message) -> CommandResult {
    let id = msg.author.id;

    if SHIPU_ID == id {
        msg.reply(ctx, t!("msg.shipu.is_shipu")).await?;
    } else {
        msg.reply(ctx, t!("msg.shipu.not_shipu")).await?;
    }

    Ok(())
}
