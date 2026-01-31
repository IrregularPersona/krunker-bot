mod bot;
use crate::bot::handler::Handler;

mod card;

use krunker_rs::Client as KrunkerClient;
use serenity::prelude::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

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
