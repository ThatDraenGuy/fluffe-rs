use rand::Rng;
use sea_orm::{
    ActiveModelTrait, EntityTrait, IntoActiveModel, PaginatorTrait, QuerySelect, Set, TryIntoModel,
};
use serenity::model::prelude::{GuildId, UserId};
use thiserror::Error;

use crate::{
    domain::femboy,
    domain::{user, Femboy, User},
    AppContext,
};

use super::{user::UserService, ServiceError};

#[derive(Error, Debug)]
pub enum FemboyError {
    #[error("Femboy already registered!")]
    AlreadyRegistered,
    #[error("No guild id found!")]
    NoGuildId,
    #[error("No femboy found!")]
    NoFemboyFound,
    #[error("No associated user found!")]
    NoUserFound,
}

pub struct FemboyService;

impl FemboyService {
    pub async fn register(
        ctx: &AppContext,
        maybe_guild_id: Option<GuildId>,
        user_id: UserId,
    ) -> Result<(), ServiceError> {
        // check if guild id is actually present
        if maybe_guild_id.is_none() {
            return Err(FemboyError::NoGuildId.into());
        }
        let guild_id = maybe_guild_id.unwrap();

        // get user id
        let user_id = UserService::find_or_create(ctx, guild_id, user_id)
            .await?
            .id;

        // check if femboy already exists
        let maybe_femboy = Femboy::find_by_id(user_id).one(&ctx.db).await?;
        if maybe_femboy.is_some() {
            return Err(FemboyError::AlreadyRegistered.into());
        };

        // create new femboy entity
        let femboy = femboy::ActiveModel {
            user_id: Set(user_id),
            ..Default::default()
        };
        Femboy::insert(femboy).exec(&ctx.db).await?;

        Ok(())
    }

    pub async fn choose(
        ctx: &AppContext,
        maybe_guild_id: Option<GuildId>,
    ) -> Result<(i64, femboy::Model, user::Model), ServiceError> {
        // check if guild id is actually present
        let guild_id = maybe_guild_id.ok_or(FemboyError::NoGuildId)?;
        let actual_server_id = guild_id.to_string();

        // count all femboys on server
        let count = Femboy::find_by_server(actual_server_id.as_str())
            .count(&ctx.db)
            .await?;

        //choose random femboy number
        let femboy_num = rand::thread_rng().gen_range(0..count);

        //fetch chosen femboy data
        let (chosen_femboy, maybe_chosen_user) = Femboy::find_by_server(actual_server_id.as_str())
            .select_also(User)
            .offset(femboy_num)
            .one(&ctx.db)
            .await?
            .ok_or(FemboyError::NoFemboyFound)?;
        let chosen_user = maybe_chosen_user.ok_or(FemboyError::NoUserFound)?;

        // get needed data
        let winnings = Self::get_femboy_win_prize();
        let wins_num = chosen_femboy.wins_num;
        let balance = chosen_femboy.balance;

        //update chosen femboy data
        let mut chosen_femboy = chosen_femboy.into_active_model();
        chosen_femboy.wins_num = Set(wins_num + 1);
        chosen_femboy.balance = Set(balance + winnings);
        let chosen_femboy = chosen_femboy.update(&ctx.db).await?.try_into_model()?;

        Ok((winnings, chosen_femboy, chosen_user))
    }

    fn get_femboy_win_prize() -> i64 {
        rand::thread_rng().gen_range(10..=20)
    }
}
