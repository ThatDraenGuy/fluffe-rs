use rust_i18n::t;
use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};

const SHIPU_ID: UserId = UserId(373748719012806656);

#[command]
async fn oleg_shipulin(ctx: &Context, msg: &Message) -> CommandResult {
    let id = msg.author.id;

    if SHIPU_ID == id {
        msg.reply(ctx, t!("shipu.is_shipu")).await?;
    } else {
        msg.reply(ctx, t!("shipu.not_shipu")).await?;
    }

    Ok(())
}
