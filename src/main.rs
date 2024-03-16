use std::sync::Arc;
use std::time::Duration;

use fluffe_rs::handlers::*;
use fluffe_rs::DbPool;
use fluffe_rs::{
    command::{self, AppCommands},
    images::{reactor::ReactorRepository, ImageRepository},
    AppResult, FluffersBot,
};
use migration::Migrator;
use migration::MigratorTrait;
use sea_orm::ConnectOptions;
use sea_orm::Database;
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

    let pool = setup_database_pool().await;

    let bot = Bot::from_env().parse_mode(ParseMode::Html);
    setup_commands(&bot)
        .await
        .expect("Couldn't setup commands!");

    let handler = dptree::entry()
        .chain(dptree::filter_async(handle_update_logging))
        .chain(dptree::filter_async(username_storage_handler))
        .branch(
            Update::filter_message().branch(
                dptree::entry()
                    .filter_command::<AppCommands>()
                    .endpoint(command::handle_command),
            ),
        );

    let image_repository: ImageRepository = ReactorRepository::default().into();
    let image_repository = Arc::new(image_repository);

    let mut dispatcher = Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![image_repository, pool])
        .enable_ctrlc_handler()
        .default_handler(handle_unhandled_update_logging)
        .build();
    info!("Bot successfully started!");
    dispatcher.dispatch().await
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

async fn setup_database_pool() -> DbPool {
    let database_url = std::env::var("DATABASE_URL").expect("No database url found!");

    let mut options = ConnectOptions::new(&database_url);
    options
        .max_connections(10)
        .min_connections(3)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Trace);

    let pool = Database::connect(options)
        .await
        .expect("Couldn't create database connection!");

    Migrator::up(&pool, None)
        .await
        .expect("Couldn't set up migrations");
    pool
}
