// external modules
use krunker_rs::Client as KrunkerClient;
use serenity::prelude::*;

// std modules
use std::sync::Arc;

// bot submodule
mod bot;
use crate::bot::handler::Handler;

// database submodule
mod database;

// verification submodule
mod verification;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // initialize database
    let pool = database::init_db().await?;
    // println!("Database initialized!");

    // env vars
    tracing::info!("Grabbing tokens...");
    let krunker_key = std::env::var("KRUNKER_KEY")?;
    let discord_token = std::env::var("DISCORD_TOKEN")?;

    // debug flags for this later pls lol
    // println!("discord token: {}", discord_token);

    let krunker_api = Arc::new(KrunkerClient::new(krunker_key)?);

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    tracing::info!("Building client...");
    let mut client = Client::builder(&discord_token, intents)
        .event_handler(Handler::new(krunker_api))
        .await
        .expect("Failure to create client");

    tracing::info!("Starting Bot...");
    if let Err(why) = client.start().await {
        eprintln!("Client error: {why:?}");
    }

    Ok(())
}
