use crate::consts::DEFAULT_LOCALE;
use crate::utils::send_error_msg;
use crate::{AppError, AppResultExt};
use crate::{AppResult, DbPool, FluffersBot};
use chrono::NaiveTime;
use chrono::TimeDelta;
use chrono::{Duration, Local};
use entity::gen::users;
use entity::{
    gen::{chats, players},
    prelude::*,
};
use rand::Rng;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, Order, QueryFilter, QueryOrder,
    QuerySelect, TransactionTrait,
};
use sea_orm::{IntoActiveModel, PaginatorTrait, Set};
use teloxide::requests::Requester;
use teloxide::types::ChatId;
use tokio::time::Duration as TokioDuration;
use tokio::time::{sleep, sleep_until};

const BATCH_SIZE: u64 = 10;
const MIN_TIME_DELTA: TimeDelta = TimeDelta::seconds(1);
const NO_JOB_TIMEOUT: TokioDuration = TokioDuration::from_secs(60 * 60);

#[derive(Clone)]
pub struct FemboyServiceCtx {
    pub db: DbPool,
    pub bot: FluffersBot,
}

pub fn initialize_femboy_service(ctx: FemboyServiceCtx) {
    tokio::spawn(schedule_femboys(ctx));
}

async fn schedule_femboys(ctx: FemboyServiceCtx) -> AppResult<()> {
    let mut time = Local::now().time();
    loop {
        // let current_time = Local::now().time();
        let batch = find_chats_batch(&ctx.db, time).await?;

        if batch.is_empty() {
            info!("No femboy selections to schedule; sleeping");
            let now = Local::now();
            let midnight = (now + Duration::days(1))
                .date_naive()
                .and_hms_opt(0, 0, 0)
                .unwrap();
            sleep(TokioDuration::from_secs(
                midnight.signed_duration_since(now).num_seconds() as u64,
            ));

            // if time == NaiveTime::MIN {
            //     // no chats to schedule; sleeping
            //     sleep(NO_JOB_TIMEOUT).await;
            // } else {
            //     time = NaiveTime::MIN;
            // }
        }

        for chat in batch.into_iter() {
            let delay = chat.femboy_time - Local::now().time();
            // immediately handle job if delay is small
            if delay > MIN_TIME_DELTA {
                info!("Scheduling next femboy selection at {}", chat.femboy_time);
                sleep(TokioDuration::from_secs(delay.num_seconds() as u64)).await;
            }
            time = chat.femboy_time;
            tokio::spawn(perform_femboy_selection(ctx.clone(), chat));
        }
    }
}

async fn find_chats_batch(db: &DbPool, time: NaiveTime) -> AppResult<Vec<chats::Model>> {
    Ok(Chats::find()
        .filter(chats::Column::FemboyTime.gt(time))
        .order_by(chats::Column::FemboyTime, Order::Asc)
        .limit(BATCH_SIZE)
        .all(db)
        .await?)
}

async fn perform_femboy_selection(ctx: FemboyServiceCtx, chat: chats::Model) -> AppResult<()> {
    info!(
        "Starting femboy selection in chat with ID [{}]",
        chat.telegram_id
    );
    let bot = ctx.bot;
    let chat_id = ChatId(chat.telegram_id);
    bot.send_message(chat_id, t!("msg.femboy.start", locale = DEFAULT_LOCALE))
        .await?;

    let t = ctx.db.begin().await?;
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
