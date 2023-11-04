mod commands;
mod domain;
mod service;

use dotenv::dotenv;
use sea_orm::{Database, DatabaseConnection};
use std::env;
use std::sync::Arc;

use serenity::async_trait;
use serenity::framework::standard::StandardFramework;

use serenity::prelude::*;

use commands::*;

#[macro_use]
extern crate rust_i18n;

i18n!("locales", fallback = "ru");

pub struct AppContext {
    db: DatabaseConnection,
}

impl TypeMapKey for AppContext {
    type Value = Arc<Mutex<AppContext>>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {}

#[tokio::main]
async fn main() {
    dotenv().ok();
    rust_i18n::set_locale("ru");

    // Get dotenv variables
    let token = env::var("DISCORD_TOKEN").expect("token");
    let connection = env::var("CONNECTION").expect("token");

    // Create framework
    let framework = StandardFramework::new().configure(|c| c.prefix("~")); // set the bot's prefix to "~"
    let framework = register_commands(framework);

    // Login with a bot token from the environment
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    // Connect to database
    let db = Database::connect(connection)
        .await
        .expect("Error connecting to database");

    // Construct app context
    let app_context = AppContext { db };

    // Setup bot data
    {
        let mut data = client.data.write().await;
        data.insert::<AppContext>(Arc::from(Mutex::from(app_context)));
    }
    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
