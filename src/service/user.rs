use sea_orm::{ActiveModelTrait, Set};
use serenity::model::prelude::{GuildId, UserId};
use thiserror::Error;

use crate::domain::{user, User};
use crate::AppContext;

use super::server::ServerService;
use super::ServiceError;

#[derive(Error, Debug)]
pub enum UserError {
    #[error("User already exists!")]
    AlreadyExists,
}

pub struct UserService;

impl UserService {
    pub async fn find(
        ctx: &AppContext,
        guild_id: GuildId,
        user_id: UserId,
    ) -> Result<Option<user::Model>, ServiceError> {
        Ok(
            User::find_by_user(guild_id.to_string(), user_id.to_string())
                .one(&ctx.db)
                .await?,
        )
    }

    #[allow(unused)]
    pub async fn register(
        ctx: &AppContext,
        guild_id: GuildId,
        user_id: UserId,
    ) -> Result<user::Model, ServiceError> {
        //if user already exists return error
        if (Self::find(ctx, guild_id, user_id).await?).is_some() {
            return Err(UserError::AlreadyExists.into());
        }

        Self::create(ctx, guild_id, user_id).await
    }

    async fn create(
        ctx: &AppContext,
        guild_id: GuildId,
        user_id: UserId,
    ) -> Result<user::Model, ServiceError> {
        let server = ServerService::find(ctx, guild_id).await?.unwrap();

        let user = user::ActiveModel {
            discord_id: Set(user_id.to_string()),
            server_id: Set(server.id),
            ..Default::default()
        };
        Ok(user.insert(&ctx.db).await?)
    }

    pub async fn find_or_create(
        ctx: &AppContext,
        guild_id: GuildId,
        user_id: UserId,
    ) -> Result<user::Model, ServiceError> {
        let maybe_user = Self::find(ctx, guild_id, user_id).await?;
        if let Some(user) = maybe_user {
            Ok(user)
        } else {
            Self::create(ctx, guild_id, user_id).await
        }
    }
}
