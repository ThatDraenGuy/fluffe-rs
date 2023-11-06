use serenity::model::prelude::GuildId;

use crate::AppContext;

use crate::domain::{server, Server};

use super::ServiceError;

pub struct ServerService;

impl ServerService {
    pub async fn find(
        ctx: &AppContext,
        guild_id: GuildId,
    ) -> Result<Option<server::Model>, ServiceError> {
        Ok(Server::find_by_actual_id(guild_id.to_string())
            .one(&ctx.db)
            .await?)
    }
}
