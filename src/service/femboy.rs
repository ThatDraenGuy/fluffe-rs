use sea_orm::{EntityTrait, Set};
use serenity::model::prelude::{GuildId, UserId};
use thiserror::Error;

use crate::{domain::femboy, domain::Femboy, AppContext};

use super::{user::UserService, ServiceError};

#[derive(Error, Debug)]
pub enum FemboyError {
    #[error("Femboy already registered!")]
    AlreadyRegistered,
    #[error("No guild id found!")]
    NoGuildId,
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
}
