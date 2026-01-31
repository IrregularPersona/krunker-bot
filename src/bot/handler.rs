// std stuff
use std::collections::HashMap;
use std::sync::Arc;

// krunker API wrapper
use krunker_rs::Client as KrunkerClient;

// serenity
// use serenity::all::{CreateEmbed, CreateMessage};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;
use sqlx::SqlitePool;

// internal
use super::commands;

#[allow(dead_code)]
pub struct Handler {
    pub krunker_api: Arc<KrunkerClient>,
    pub pool: SqlitePool,
    pub commands: HashMap<String, Arc<dyn commands::KrunkerCommand>>,
}

impl Handler {
    pub fn new(krunker_api: Arc<KrunkerClient>, pool: SqlitePool) -> Self {
        let mut commands_map = HashMap::new();

        for cmd in commands::all_commands() {
            let meta = cmd.metadata();
            commands_map.insert(meta.name.to_string(), Arc::clone(&cmd));
            for alias in meta.aliases {
                commands_map.insert(alias.to_string(), Arc::clone(&cmd));
            }
        }

        Self {
            krunker_api,
            pool,
            commands: commands_map,
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        if !msg.content.starts_with("&") {
            return;
        }

        tracing::info!(
            user = %msg.author.name,
            user_id = %msg.author.id,
            command = %msg.content,
            "Command received"
        );

        let content = &msg.content[1..];
        let mut parts = content.split_whitespace();
        let command = parts.next().unwrap_or("");
        let args: Vec<&str> = parts.collect();

        if let Some(cmd) = self.commands.get(command) {
            if let Err(why) = cmd.execute(&ctx, &msg, &self.krunker_api, args, &self.pool).await {
                tracing::error!("Error executing command {}: {:?}", command, why);
                let _ = msg.channel_id.say(&ctx.http, format!("Error: {}", why)).await;
            }
        } else {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Not a valid command!").await {
                tracing::error!("Error sending message: {why:?}");
            }
        }
    }
}
