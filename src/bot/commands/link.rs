use async_trait::async_trait;
use krunker_rs::Client as KrunkerClient;
use serenity::model::channel::Message;
use serenity::prelude::*;
use sqlx::SqlitePool;
use std::sync::Arc;

use super::{CommandMetadata, KrunkerCommand};

pub struct Link;

#[async_trait]
impl KrunkerCommand for Link {
    fn metadata(&self) -> CommandMetadata {
        CommandMetadata {
            name: "link",
            description: "Link your Krunker account to your Discord account",
            usage: "&link <username>",
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
        msg.channel_id.say(&ctx.http, "Account linking is not yet implemented.").await?;
        Ok(())
    }
}
