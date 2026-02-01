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

    async fn execute(
        &self,
        ctx: &Context,
        msg: &Message,
        _krunker_api: &Arc<KrunkerClient>,
        args: Vec<&str>,
        pool: &SqlitePool,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let username = match args.get(0) {
            Some(u) if !u.is_empty() => u,
            _ => {
                msg.channel_id
                    .say(&ctx.http, "Usage: &link <krunker_username>")
                    .await?;
                return Ok(());
            }
        };

        let discord_id = msg.author.id.to_string();

        match crate::verification::flow::start_verification(pool, &discord_id, username).await {
            Ok(code) => {
                let response = format!(
                    "Verification started for **{}**!\n\n\
                    Please post the following code to your Krunker.io social profile:\n\
                    `{}`\n\n\
                    After posting, run `&verify` to complete the link.",
                    username, code
                );
                msg.channel_id.say(&ctx.http, response).await?;
            }
            Err(e) => {
                msg.channel_id
                    .say(&ctx.http, format!("Error: {}", e))
                    .await?;
            }
        }

        Ok(())
    }
}
