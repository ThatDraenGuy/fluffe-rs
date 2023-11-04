use serenity::{
    framework::standard::{
        macros::{command, group},
        CommandResult,
    },
    model::prelude::*,
    prelude::*,
};

#[group]
#[commands(femboy_register, femboy_leaderboard, femboy)]
pub struct FEMBOY;

#[command]
async fn femboy_register(_ctx: &Context, _msg: &Message) -> CommandResult {
    todo!()
}

#[command]
async fn femboy_leaderboard(_ctx: &Context, _msg: &Message) -> CommandResult {
    todo!()
}

#[command]
async fn femboy(_ctx: &Context, _msg: &Message) -> CommandResult {
    todo!()
}
