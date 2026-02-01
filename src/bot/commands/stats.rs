use async_trait::async_trait;
use krunker_rs::Client as KrunkerClient;
use serenity::all::{CreateEmbed, CreateMessage};
use serenity::model::channel::Message;
use serenity::prelude::*;
use sqlx::SqlitePool;
use std::sync::Arc;

use super::{CommandMetadata, KrunkerCommand};

pub struct Stats;

#[async_trait]
impl KrunkerCommand for Stats {
    fn metadata(&self) -> CommandMetadata {
        CommandMetadata {
            name: "stats",
            description: "Show general player statistics (K/D, Level, KR)",
            usage: "&stats <username>",
            aliases: &["p"],
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
        let username = args.get(0).unwrap_or(&"");

        if username.is_empty() {
            msg.channel_id
                .say(&ctx.http, "Usage: &stats <username>")
                .await?;
            return Ok(());
        }

        match krunker_api.get_player(username).await {
            Ok(player) => {
                let embed = CreateEmbed::new()
                    .title(format!(
                        "{}{}",
                        player.player_name,
                        if player.player_verified { " ✅" } else { "" }
                    ))
                    .field(
                        "Clan",
                        if player.player_clan.is_empty() {
                            "None"
                        } else {
                            &player.player_clan
                        },
                        true,
                    )
                    .field("Level", player.player_level.to_string(), true)
                    .field("KR", player.player_kr.to_string(), true)
                    .field("K/D Ratio", format!("{:.2}", player.player_kdr), true)
                    .field("Games Played", player.player_games.to_string(), true)
                    .color(0x00ff00);

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
