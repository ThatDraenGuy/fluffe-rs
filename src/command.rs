use std::sync::Arc;

use sea_orm::{ActiveModelTrait, IntoActiveModel, Set, TransactionTrait};
use teloxide::{
    net::Download,
    payloads::{SendAnimationSetters, SendMessageSetters, SendStickerSetters},
    requests::{Requester, ResponseResult},
    types::{InputFile, Me, Message},
    utils::command::BotCommands,
};

use crate::{
    built_info,
    consts::{DEFAULT_MENTION, SHIPU_STICKER},
    images::{pet_gif_creator::create_pet_gif, ImageRepository, ImageRepositoryTrait},
    utils::*,
    AppError, AppResult, ClientError, DbPool, FluffersBot,
};

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "snake_case")]
pub enum AppCommands {
    // GetFurry, // temporarily disabled
    Pet(String),
    MyStats,
    TopPets,
    Shipu,
    About,
}

pub async fn handle_command(
    // image_repository: Arc<ImageRepository>,
    me: Me,
    bot: FluffersBot,
    msg: Message,
    cmd: AppCommands,
    db: DbPool,
) -> ResponseResult<()> {
    let user_id = msg.from().map_or(0, |u| u.id.0);
    let chat_id = msg.chat.id;

    let result = match cmd.clone() {
        // AppCommands::GetFurry => get_furry(image_repository, &bot, &msg).await,
        AppCommands::Pet(arg) => pet(&db, &bot, &me, &msg, &arg).await,
        AppCommands::MyStats => my_stats(&db, &bot, &msg).await,
        AppCommands::TopPets => top_pets(&db, &bot, &msg).await,
        AppCommands::Shipu => shipu(&bot, &msg).await,
        AppCommands::About => about(&bot, &msg).await,
    };

    match result {
        Ok(_) => log::info!("Handled command /{cmd:?}: from user [{user_id}] in chat [{chat_id}]",),
        Err(e) => {
            if let AppError::ClientError(cli_err) = &e {
                log::warn!(
                    "Client error {cli_err:?}: on command /{cmd:?}: from user [{user_id}] in chat [{chat_id}]"
                );
            } else {
                log::error!(
                    "Server error {e:?}: on command /{cmd:?}: from user [{user_id}] in chat [{chat_id}]"
                );
            }

            match &e {
                AppError::ClientError(cli_err) => match cli_err {
                    ClientError::NoUser(username) => {
                        bot.send_message(
                            msg.chat.id,
                            t!(
                                "msg.common.error.client.unknown_username",
                                locale = get_language_code(&msg),
                                mention = username,
                            ),
                        )
                        .reply_to_message_id(msg.id)
                        .await?;
                    }
                    ClientError::NoMention(command) => {
                        bot.send_message(
                            msg.chat.id,
                            t!(
                                "msg.common.error.client.mention_argument",
                                command = command,
                                locale = get_language_code(&msg),
                                mention = DEFAULT_MENTION,
                            ),
                        )
                        .reply_to_message_id(msg.id)
                        .await?;
                    }
                },
                AppError::UnknownPlayer => {
                    bot.send_message(
                        msg.chat.id,
                        t!(
                            "msg.common.error.server.unknown_player",
                            locale = get_language_code(&msg),
                        ),
                    )
                    .reply_to_message_id(msg.id)
                    .await?;
                }
                AppError::Database(db_err) => {
                    bot.send_message(
                        msg.chat.id,
                        t!(
                            "msg.common.error.server.db_err",
                            locale = get_language_code(&msg),
                            msg = db_err
                        ),
                    )
                    .reply_to_message_id(msg.id)
                    .await?;
                }
                e => {
                    bot.send_message(
                        msg.chat.id,
                        t!(
                            "msg.common.error.server.unknown_err",
                            locale = get_language_code(&msg),
                            msg = e
                        ),
                    )
                    .reply_to_message_id(msg.id)
                    .await?;
                }
            }
        }
    }

    Ok(())
}

#[allow(unused)]
async fn get_furry(
    image_repository: Arc<ImageRepository>,
    bot: &FluffersBot,
    msg: &Message,
) -> AppResult<()> {
    let image = image_repository.get_random_image().await?;
    bot.send_photo(msg.chat.id, image).await?;
    Ok(())
}

async fn pet(
    db: &DbPool,
    bot: &FluffersBot,
    me: &Me,
    msg: &Message,
    mention: &str,
) -> AppResult<()> {
    let username = resolve_mention(mention, "pet")?;

    if me.username() == username {
        // NO U
        bot.send_message(
            msg.chat.id,
            t!("msg.pet.error.me", locale = get_language_code(msg)),
        )
        .reply_to_message_id(msg.id)
        .await?;
        return Ok(());
    }
    let t = db.begin().await?;

    let (target_player, target_user) = find_player_by_username(&t, msg.chat.id, username).await?;
    let (source_player, source_user) = find_msg_player(&t, msg).await?;

    if target_user.eq_by_id(&source_user) {
        // SELF PET
        bot.send_message(
            msg.chat.id,
            t!(
                "msg.pet.error.self",
                locale = get_language_code(msg),
                mention = mention,
            ),
        )
        .reply_to_message_id(msg.id)
        .await?;
        return Ok(());
    }

    let target_user_id = target_user.get_telegram_id();

    let user_photos = bot.get_user_profile_photos(target_user_id).await?;
    let Some(photo) = user_photos.photos.first().and_then(|photo| photo.first()) else {
        bot.send_message(
            msg.chat.id,
            t!(
                "msg.pet.error.no_photo",
                locale = get_language_code(msg),
                mention = mention,
            ),
        )
        .reply_to_message_id(msg.id)
        .await?;
        return Ok(());
    };

    let file = bot.get_file(&photo.file.id).await?;

    let mut avatar = Vec::with_capacity(file.size as usize);
    bot.download_file(&file.path, &mut avatar).await?;

    let gif = create_pet_gif(avatar, username)?;

    let gif_msg = bot
        .send_animation(msg.chat.id, gif)
        .reply_to_message_id(msg.id)
        .await?;

    let mut source_player = source_player.into_active_model();
    let mut target_player = target_player.into_active_model();

    source_player.pets_given = Set(source_player.pets_given.as_ref() + 1);
    target_player.pets_received = Set(target_player.pets_received.as_ref() + 1);

    bot.send_message(
        msg.chat.id,
        t!(
            "msg.pet.success",
            locale = get_language_code(msg),
            target = format!("@{username}"),
            num = target_player.pets_received.as_ref()
        ),
    )
    .reply_to_message_id(gif_msg.id)
    .await?;

    source_player.save(&t).await?;
    target_player.save(&t).await?;
    t.commit().await?;

    Ok(())
}

async fn my_stats(db: &DbPool, bot: &FluffersBot, msg: &Message) -> AppResult<()> {
    let (player, user) = find_msg_player(db, msg).await?;

    bot.send_message(
        msg.chat.id,
        t!(
            "msg.my_stats.success",
            locale = get_language_code(msg),
            target = user.mention().as_deref().unwrap_or("Someone"),
            pets_received = player.pets_received,
            pets_given = player.pets_given,
            coins = player.coins
        ),
    )
    .reply_to_message_id(msg.id)
    .await?;

    Ok(())
}

async fn top_pets(db: &DbPool, bot: &FluffersBot, msg: &Message) -> AppResult<()> {
    let top_received = find_top_pets_received(db, msg.chat.id).await?;
    let top_given = find_top_pets_given(db, msg.chat.id).await?;

    format_as_top_list(&top_received, |player| player.pets_received.to_string());

    bot.send_message(
        msg.chat.id,
        t!(
            "msg.top_pets.success",
            locale = get_language_code(msg),
            received_list =
                format_as_top_list(&top_received, |player| player.pets_received.to_string()),
            given_list = format_as_top_list(&top_given, |player| player.pets_given.to_string())
        ),
    )
    .reply_to_message_id(msg.id)
    .await?;

    Ok(())
}

async fn shipu(bot: &FluffersBot, msg: &Message) -> AppResult<()> {
    bot.send_sticker(msg.chat.id, InputFile::memory(&SHIPU_STICKER as &[u8]))
        .reply_to_message_id(msg.id.0)
        .await?;

    Ok(())
}

async fn about(bot: &FluffersBot, msg: &Message) -> AppResult<()> {
    let last_update = built::util::strptime(built_info::BUILT_TIME_UTC).format("%d-%m-%Y %H:%M:%S");

    bot.send_message(
        msg.chat.id,
        t!(
            "msg.about.info",
            locale = get_language_code(msg),
            version = built_info::PKG_VERSION,
            authors = built_info::PKG_AUTHORS,
            last_update = last_update
        ),
    )
    .reply_to_message_id(msg.id)
    .await?;

    Ok(())
}
