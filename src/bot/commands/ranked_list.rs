use async_trait::async_trait;
use krunker_rs::Client as KrunkerClient;
use serenity::all::{CreateEmbed, CreateMessage};
use serenity::model::channel::Message;
use serenity::prelude::*;
use sqlx::SqlitePool;
use std::sync::Arc;

use super::{CommandMetadata, KrunkerCommand};

pub struct RankedList;

#[async_trait]
impl KrunkerCommand for RankedList {
    fn metadata(&self) -> CommandMetadata {
        CommandMetadata {
            name: "rankedlist",
            description: "List match IDs for the last N ranked matches",
            usage: "&rl <username> [count]",
            aliases: &["rl"],
        }
    }

    async fn execute(
        &self,
        ctx: &Context,
        msg: &Message,
        krunker_api: &Arc<KrunkerClient>,
        args: Vec<&str>,
        _pool: &SqlitePool,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let username = args.get(0).unwrap_or(&"");
        if username.is_empty() {
            msg.channel_id
                .say(&ctx.http, "Usage: &rl <username> [count]")
                .await?;
            return Ok(());
        }

        let count_str = args.get(1).unwrap_or(&"");
        let mut count: i32 = 1;
        if !count_str.is_empty() {
            count = count_str.parse::<i32>().unwrap_or(1);
        }

        match krunker_api.get_player_matches(username, None, None).await {
            Ok(data) => {
                let matches = data.pmr_matches.unwrap_or_default();
                if matches.is_empty() {
                    msg.channel_id
                        .say(&ctx.http, "No recent ranked data found!")
                        .await?;
                    return Ok(());
                }

                let mut embed = CreateEmbed::new()
                    .title(format!("Recent Ranked Match IDs"))
                    .color(0x0000ff);

                for (i, pmatch) in matches.iter().take(count as usize).enumerate() {
                    embed = embed.field(
                        format!("Match #{}", i + 1),
                        pmatch.pm_match_id.to_string(),
                        false,
                    );
                }

                msg.channel_id
                    .send_message(&ctx.http, CreateMessage::new().embed(embed))
                    .await?;
            }
            Err(e) => {
                let response = format!("Error fetching stats: {}", e);
                msg.channel_id.say(&ctx.http, response).await?;
            }
        }
        Ok(())
    }
}
