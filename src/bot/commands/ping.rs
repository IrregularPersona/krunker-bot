use async_trait::async_trait;
use krunker_rs::Client as KrunkerClient;
use serenity::model::channel::Message;
use serenity::prelude::*;
use sqlx::SqlitePool;
use std::sync::Arc;

use super::{CommandMetadata, KrunkerCommand};

pub struct Ping;

#[async_trait]
impl KrunkerCommand for Ping {
    fn metadata(&self) -> CommandMetadata {
        CommandMetadata {
            name: "ping",
            description: "Check if the bot is responsive",
            usage: "&ping",
            aliases: &[],
        }
    }

    #[allow(unused_variables)]
    async fn execute(
        &self,
        ctx: &Context,
        msg: &Message,
        krunker_api: &Arc<KrunkerClient>,
        args: Vec<&str>,
        _pool: &SqlitePool,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        msg.channel_id.say(&ctx.http, "ping back").await?;
        Ok(())
    }
}
