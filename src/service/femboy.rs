use std::collections::HashMap;

use crate::consts::DEFAULT_LOCALE;
use crate::utils::send_error_msg;
use crate::{AppError, AppResultExt};
use crate::{AppResult, DbPool, FluffersBot};
use chrono::Timelike;
use entity::gen::users;
use entity::{
    gen::{chats, players},
    prelude::*,
};
use rand::Rng;
use sea_orm::{
    ActiveModelTrait, ConnectionTrait, EntityTrait, QueryOrder, QuerySelect, TransactionTrait,
};
use sea_orm::{IntoActiveModel, PaginatorTrait, Set};
use teloxide::requests::Requester;
use teloxide::types::ChatId;
use tokio::task::JoinHandle;
use tokio_schedule::{every, Job};

type ScheduledGames = HashMap<ChatId, JoinHandle<()>>;

pub struct FemboyService {
    scheduled_games: ScheduledGames,
    ctx: FemboyServiceCtx,
}
impl FemboyService {
    pub async fn init(ctx: FemboyServiceCtx) -> AppResult<Self> {
        Ok(Self {
            scheduled_games: Self::schedule_games(&ctx).await?,
            ctx,
        })
    }

    async fn schedule_games(ctx: &FemboyServiceCtx) -> AppResult<ScheduledGames> {
        let chats = Chats::find().all(&ctx.db).await?;
        let mut scheduled_games = ScheduledGames::new();
        for chat in chats.into_iter() {
            let ctx_bind = ctx.clone();
            let scheduled = every(1)
                .day()
                .at(
                    chat.femboy_time.hour(),
                    chat.femboy_time.minute(),
                    chat.femboy_time.second(),
                )
                .perform(move || {
                    let ctx_bind = ctx_bind.clone();
                    async move {
                        perform_femboy_selection(ctx_bind.clone(), ChatId(chat.telegram_id)).await;
                    }
                });
            scheduled_games.insert(ChatId(chat.telegram_id), tokio::spawn(scheduled));
        }
        todo!()
    }

    pub fn update_chat_schedule(&self, chat: chats::Model) {
        todo!()
    }
}

#[derive(Clone)]
pub struct FemboyServiceCtx {
    pub db: DbPool,
    pub bot: FluffersBot,
}

async fn perform_femboy_selection(ctx: FemboyServiceCtx, telegram_id: ChatId) -> AppResult<()> {
    info!(
        "Starting femboy selection in chat with ID [{}]",
        telegram_id
    );
    let t = ctx.db.begin().await?;
    let chat = Chats::find_by_telegram_id(telegram_id)
        .one(&t)
        .await?
        .ok_or(AppError::UnknownChat)?;
    let bot = ctx.bot;
    let chat_id = ChatId(chat.telegram_id);
    bot.send_message(chat_id, t!("msg.femboy.start", locale = DEFAULT_LOCALE))
        .await?;

    match choose_femboy(&t, chat).await {
        Err(e) => send_error_msg(&bot, chat_id, DEFAULT_LOCALE, None, &e).await?,
        Ok(winner) => {
            bot.send_message(
                chat_id,
                t!(
                    "msg.femboy.win",
                    locale = DEFAULT_LOCALE,
                    mention = winner.1.mention().as_deref().unwrap_or("Someone"),
                    win_amount = winner.0.femboy_wins
                ),
            )
            .await?;
        }
    };
    t.commit().await?;

    Ok(())
}

async fn choose_femboy(
    conn: &impl ConnectionTrait,
    chat: chats::Model,
) -> AppResult<(players::Model, users::Model)> {
    let players_count = Players::find_by_chat(chat.id)
        .order_by_asc(players::Column::Id)
        .count(conn)
        .await?;

    let winner_index = get_random_num(0, players_count);

    let winner = Players::find_by_chat(chat.id)
        .find_also_related(Users)
        .order_by_asc(players::Column::Id)
        .offset(winner_index)
        .one(conn)
        .await?
        .ok_or(AppError::UnknownPlayer)
        .map_tuple_with_option(AppError::UnknownUser)?;

    let mut winner_player = winner.0.into_active_model();
    winner_player.femboy_wins = Set(winner_player.femboy_wins.as_ref() + 1);

    Ok((winner_player.update(conn).await?, winner.1))
}

fn get_random_num(min: u64, max: u64) -> u64 {
    let mut rng = rand::thread_rng();
    rng.gen_range(min..max)
}
