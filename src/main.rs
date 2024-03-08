use std::sync::Arc;

use fluffe_rs::{
    command::{self, AppCommands},
    image::{reactor::ReactorRepository, ImageRepository},
    AppResult, FluffersBot,
};
use teloxide::{prelude::*, types::ParseMode, utils::command::BotCommands};

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

#[macro_use]
extern crate rust_i18n;

i18n!("locales", fallback = "ru");

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    pretty_env_logger::init();

    info!("Starting bot!");

    let bot = Bot::from_env().parse_mode(ParseMode::Html);
    setup_commands(&bot)
        .await
        .expect("Couldn't setup commands!");

    let handler = dptree::entry().branch(
        Update::filter_message().branch(
            dptree::entry()
                .filter_command::<AppCommands>()
                .endpoint(command::handle_command),
        ),
    );

    let image_repository: ImageRepository = ReactorRepository::default().into();
    let image_repository = Arc::new(image_repository);

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![image_repository])
        .enable_ctrlc_handler()
        .default_handler(default_log_handler)
        .build()
        .dispatch()
        .await;
}

async fn setup_commands(bot: &FluffersBot) -> AppResult<()> {
    let locales = available_locales!();
    for locale in locales.into_iter() {
        let mut commands = AppCommands::bot_commands();

        for command in commands.iter_mut() {
            command.description =
                t!(&format!("commands.{}", command.command), locale = locale).to_string();
        }

        bot.set_my_commands(commands).language_code(locale).await?;
    }

    Ok(())
}

async fn default_log_handler(upd: Arc<Update>) {
    let update_id = upd.id;
    if let Some(user) = upd.user() {
        let user_id = user.id;
        if let Some(chat) = upd.chat() {
            let chat_id = chat.id;
            warn!("Unhandled update [{update_id}]: user: [{user_id}] chat: [{chat_id}]");
        } else {
            warn!("Unhandled update [{update_id}]: user: [{user_id}] ");
        };
    } else if let Some(chat) = upd.chat() {
        let chat_id = chat.id;
        warn!("Unhandled update [{update_id}]: chat: [{chat_id}]");
    } else {
        warn!("Unhandled update [{update_id}]: kind: {:?}", upd.kind);
    }
}
