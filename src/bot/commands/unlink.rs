use async_trait::async_trait;
use krunker_rs::Client as KrunkerClient;
use serenity::model::channel::Message;
use serenity::prelude::*;
use sqlx::SqlitePool;
use std::sync::Arc;

use super::{CommandMetadata, KrunkerCommand};
use crate::database::queries;

pub struct Unlink;

#[async_trait]
impl KrunkerCommand for Unlink {
    fn metadata(&self) -> CommandMetadata {
        CommandMetadata {
            name: "unlink",
            description: "Unlink your Krunker account from your Discord account",
            usage: "&unlink",
            aliases: &[],
        }
    }

    async fn execute(
        &self,
        ctx: &Context,
        msg: &Message,
        _krunker_api: &Arc<KrunkerClient>,
        _args: Vec<&str>,
        pool: &SqlitePool,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let discord_id = msg.author.id.to_string();

        if !queries::user_exists(pool, &discord_id).await? {
            msg.channel_id
                .say(&ctx.http, "You are not linked to any Krunker account.")
                .await?;
            return Ok(());
        }

        queries::delete_user(pool, &discord_id).await?;

        msg.channel_id
            .say(
                &ctx.http,
                "✅ Successfully unlinked your account. You can now link a new one with `&link <username>`.",
            )
            .await?;

        Ok(())
    }
}
