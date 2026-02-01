// external modules
use krunker_rs::Client as KrunkerClient;
use serenity::prelude::*;
use std::sync::Arc;

// internal bot modules
mod bot;
mod database;
mod verification;

use crate::bot::commands;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // initialize database
    let pool = database::init_db().await?;

    // env vars
    tracing::info!("Grabbing tokens...");
    let krunker_key = std::env::var("KRUNKER_API")?;
    let discord_token = std::env::var("DISCORD_TOKEN")?;
    let guild_id = std::env::var("GUILD_ID")
        .ok()
        .and_then(|id| id.parse::<u64>().ok())
        .map(serenity::model::id::GuildId::new);

    let krunker_api = Arc::new(KrunkerClient::new(krunker_key)?);

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    tracing::info!("Building Poise framework...");
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands::all_commands(),
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("&".into()),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                tracing::info!("Registering commands...");
                if let Some(guild_id) = guild_id {
                    tracing::info!("Registering commands to guild: {}", guild_id);
                    poise::builtins::register_in_guild(ctx, &framework.options().commands, guild_id).await?;
                } else {
                    tracing::info!("Registering commands globally");
                    poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                }
                
                Ok(commands::Data {
                    krunker_api,
                    pool,
                })
            })
        })
        .build();

    tracing::info!("Starting Bot...");
    let mut client = Client::builder(&discord_token, intents)
        .framework(framework)
        .await
        .expect("Failure to create client");

    if let Err(why) = client.start().await {
        tracing::error!("Client error: {why:?}");
    }

    Ok(())
}
