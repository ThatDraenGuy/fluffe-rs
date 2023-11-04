use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter, Set};
use serenity::model::prelude::{GuildId, UserId};
use thiserror::Error;

use crate::{
    domain::{femboy, server, user, Femboy, Server, User},
    AppContext,
};

#[derive(Error, Debug)]
pub enum FemboyError {
    #[error("Femboy already registered!")]
    AlreadyRegistered,
    #[error("No guild id found!")]
    NoGuildId,
    #[error(transparent)]
    DbErr(#[from] DbErr),
}

pub struct FemboyService;

impl FemboyService {
    pub async fn register(
        ctx: &AppContext,
        maybe_guild_id: Option<GuildId>,
        user_id: UserId,
    ) -> Result<(), FemboyError> {
        // check if guild id is actually present
        if maybe_guild_id.is_none() {
            return Err(FemboyError::NoGuildId);
        }
        let guild_id = maybe_guild_id.unwrap();

        // get user entity
        let (server, maybe_user) = Server::find()
            .filter(server::Column::ActualId.eq(guild_id.to_string()))
            .find_also_related(User)
            .one(&ctx.db)
            .await?
            .unwrap();

        // get user id or create user entity in case it didn't exist and get id that way
        let user_id = if let Some(u) = maybe_user {
            Ok::<i64, DbErr>(u.id)
        } else {
            let user = user::ActiveModel {
                discord_id: Set(user_id.to_string()),
                server_id: Set(server.id),
                ..Default::default()
            };
            Ok(User::insert(user).exec(&ctx.db).await?.last_insert_id)
        }?;

        // check if femboy already exists
        let maybe_femboy = Femboy::find()
            .filter(femboy::Column::UserId.eq(user_id))
            .one(&ctx.db)
            .await?;
        if maybe_femboy.is_some() {
            return Err(FemboyError::AlreadyRegistered);
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
