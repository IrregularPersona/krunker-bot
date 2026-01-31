// std stuff
use std::collections::HashMap;
use std::sync::Arc;

// krunker API wrapper
use krunker_rs::Client as KrunkerClient;

// serenity
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;

// internal
use super::commands::{self, KrunkerCommand};

#[allow(dead_code)]
pub struct Handler {
    pub krunker_api: Arc<KrunkerClient>,
    pub commands: HashMap<String, Box<dyn commands::KrunkerCommand + Send + Sync>>,
}

impl Handler {
    pub fn new(krunker_api: Arc<KrunkerClient>) -> Self {
        let mut commands: HashMap<String, Box<dyn KrunkerCommand + Send + Sync>> = HashMap::new();

        commands.insert("ping".to_string(), Box::new(commands::Ping));
        commands.insert("p".to_string(), Box::new(commands::Stats));
        commands.insert("r".to_string(), Box::new(commands::RankedStats));
        commands.insert("rl".to_string(), Box::new(commands::RankedList));
        commands.insert("sm".to_string(), Box::new(commands::SpecificMatch));

        commands.insert("help".to_string(), Box::new(commands::Help));
        commands.insert("h".to_string(), Box::new(commands::Help));

        Self {
            krunker_api,
            commands,
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
            cmd.execute(&ctx, &msg, &self.krunker_api, args).await;
        } else {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Not a valid command!").await {
                tracing::error!("Error sending message: {why:?}");
            }
        }
    }
}
