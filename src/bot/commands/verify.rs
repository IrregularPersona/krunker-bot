use async_trait::async_trait;
use krunker_rs::Client as KrunkerClient;
use serenity::model::channel::Message;
use serenity::prelude::*;
use sqlx::SqlitePool;
use std::sync::Arc;

use super::{CommandMetadata, KrunkerCommand};

pub struct Verify;

#[async_trait]
impl KrunkerCommand for Verify {
    fn metadata(&self) -> CommandMetadata {
        CommandMetadata {
            name: "verify",
            description: "Verify your linked Krunker account",
            usage: "&verify",
            aliases: &[],
        }
    }

    async fn execute(
        &self,
        ctx: &Context,
        msg: &Message,
        krunker_api: &Arc<KrunkerClient>,
        _args: Vec<&str>,
        pool: &SqlitePool,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use crate::verification::flow::{check_verification, complete_verification, VerificationResult};

        let discord_id = msg.author.id.to_string();

        match check_verification(pool, krunker_api, &discord_id).await {
            Ok(VerificationResult::Success { krunker_username }) => {
                complete_verification(pool, &discord_id, &krunker_username).await?;
                msg.channel_id
                    .say(
                        &ctx.http,
                        format!(
                            "✅ Successfully verified! Your Discord account is now linked to **{}**.",
                            krunker_username
                        ),
                    )
                    .await?;
            }
            Ok(VerificationResult::NotFound {
                code,
                krunker_username,
                attempts,
            }) => {
                let response = format!(
                    "❌ Verification code not found for **{}**.\n\n\
                    Make sure you've posted this exactly: `{}`\n\
                    Attempts: {}/5",
                    krunker_username, code, attempts
                );
                msg.channel_id.say(&ctx.http, response).await?;
            }
            Ok(VerificationResult::NoVerification) => {
                msg.channel_id
                    .say(
                        &ctx.http,
                        "You don't have an active verification session. Use `&link <username>` first.",
                    )
                    .await?;
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
