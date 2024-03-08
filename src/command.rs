use std::sync::Arc;

use teloxide::{
    requests::{Requester, ResponseResult},
    types::Message,
    utils::command::BotCommands,
};

use crate::{
    image::{ImageRepository, ImageRepositoryTrait},
    AppResult, FluffersBot,
};

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "snake_case")]
pub enum AppCommands {
    GetFurry,
}

pub async fn handle_command(
    image_repository: Arc<ImageRepository>,
    bot: FluffersBot,
    msg: Message,
    cmd: AppCommands,
) -> ResponseResult<()> {
    let user_id = msg.from().map_or(0, |u| u.id.0);
    let chat_id = msg.chat.id;

    let result = match cmd {
        AppCommands::GetFurry => get_furry(image_repository, bot, &msg).await,
    };

    match result {
        Ok(_) => log::info!("Handled command /{cmd:?}: from user [{user_id}] in chat [{chat_id}]",),
        Err(e) => log::error!(
            "Error {e:?}: on command /{cmd:?}: from user [{user_id}] in chat [{chat_id}]",
        ),
    }

    Ok(())
}

async fn get_furry(
    image_repository: Arc<ImageRepository>,
    bot: FluffersBot,
    msg: &Message,
) -> AppResult<()> {
    let image = image_repository.get_random_image().await?;
    bot.send_photo(msg.chat.id, image).await?;
    Ok(())
}
