use async_trait::async_trait;
use krunker_rs::Client as KrunkerClient;
use serenity::all::{CreateEmbed, CreateMessage};
use serenity::model::channel::Message;
use serenity::prelude::*;
use sqlx::SqlitePool;
use std::sync::Arc;

use super::{CommandMetadata, KrunkerCommand};

pub struct RankedStats;

#[async_trait]
impl KrunkerCommand for RankedStats {
    fn metadata(&self) -> CommandMetadata {
        CommandMetadata {
            name: "rankedstats",
            description: "Show detailed stats for the last N ranked matches",
            usage: "&rankedstats <username> [count]",
            aliases: &["r"],
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
        let count_str = args.get(1).unwrap_or(&"");

        if username.is_empty() {
            msg.channel_id
                .say(&ctx.http, "Usage: &rankedstats <username> [count]")
                .await?;
            return Ok(());
        }

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
                    .title(format!("Recent Ranked Matches - {}", username))
                    .color(0x00ff00);

                for (_i, pmatch) in matches.iter().take(count as usize).enumerate() {
                    let kdr = if pmatch.pm_deaths > 0 {
                        pmatch.pm_kills as f64 / pmatch.pm_deaths as f64
                    } else {
                        pmatch.pm_kills as f64
                    };

                    let result = if pmatch.pm_victory == 1 {
                        "✅ Victory"
                    } else {
                        "❌ Defeat"
                    };

                    let match_info = format!(
                        "{}\n\
                      K/D: {}/{} ({:.2})\n\
                      Score: {} | Assists: {}\n\
                      Accuracy: {}%",
                        result,
                        pmatch.pm_kills,
                        pmatch.pm_deaths,
                        kdr,
                        pmatch.pm_score,
                        pmatch.pm_assists,
                        pmatch.pm_accuracy,
                    );

                    embed = embed.field(
                        format!("Match #{} - {}", pmatch.pm_match_id, pmatch.pm_date),
                        match_info,
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
