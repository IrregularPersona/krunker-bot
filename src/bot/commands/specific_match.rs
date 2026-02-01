use async_trait::async_trait;
use chrono::TimeDelta;
use krunker_rs::Client as KrunkerClient;
use krunker_rs::MatchParticipant;
use serenity::all::{CreateEmbed, CreateMessage};
use serenity::model::channel::Message;
use serenity::prelude::*;
use sqlx::SqlitePool;
use std::sync::Arc;

use super::{CommandMetadata, KrunkerCommand};

pub struct SpecificMatch;

#[async_trait]
impl KrunkerCommand for SpecificMatch {
    fn metadata(&self) -> CommandMetadata {
        CommandMetadata {
            name: "specificmatch",
            description: "Get detailed statistics for a specific match ID",
            usage: "&sm <match_id>",
            aliases: &["sm"],
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
        let id_str = args.get(0).unwrap_or(&"");
        let match_id = id_str.parse::<i64>().unwrap_or(0);

        if match_id <= 0 {
            msg.channel_id.say(&ctx.http, "Usage: &sm <match_id>").await?;
            return Ok(());
        }

        match krunker_api.get_match(match_id).await {
            Ok(data) => {
                let participants = match data.match_participants {
                    Some(ref p) if !p.is_empty() => p,
                    _ => {
                        msg.channel_id.say(&ctx.http, "No participants found for this match.").await?;
                        return Ok(());
                    }
                };

                let dur = TimeDelta::milliseconds(data.match_duration as i64);
                let mins = dur.num_minutes();
                let secs = dur.num_seconds() % 60;

                let mut embed = CreateEmbed::new()
                    .title(format!("Match Details - ID: {}", data.match_id))
                    .field("Map", data.match_map.to_string(), true)
                    .field("Duration", format!("{}m {}s", mins, secs), true)
                    .field("Date", &data.match_date, true)
                    .color(0x00ff00);

                let mut team_1: Vec<&MatchParticipant> = Vec::new();
                let mut team_2: Vec<&MatchParticipant> = Vec::new();

                for participant in participants {
                    if participant.mp_team == 1 {
                        team_1.push(participant);
                    } else {
                        team_2.push(participant);
                    }
                }

                let format_player = |p: &MatchParticipant| -> String {
                    let kda = format!("{}/{}/{}", p.mp_kills, p.mp_deaths, p.mp_assists);
                    let result = if p.mp_victory == 1 { "🏆" } else { "" };
                    format!(
                        "**{}** {}\nK/D/A: {} | Score: {}\nDamage: {} | Obj: {}",
                        p.mp_player_name,
                        result,
                        kda,
                        p.mp_score,
                        p.mp_damage_done,
                        p.mp_objective_score
                    )
                };

                if !team_1.is_empty() {
                    let team_1_stats = team_1
                        .iter()
                        .map(|p| format_player(p))
                        .collect::<Vec<_>>()
                        .join("\n\n");

                    embed = embed.field(
                        format!(
                            "Team 1 {}",
                            if team_1[0].mp_victory == 1 { "🏆" } else { "" }
                        ),
                        team_1_stats,
                        false,
                    );
                }

                if !team_2.is_empty() {
                    let team_2_stats = team_2
                        .iter()
                        .map(|p| format_player(p))
                        .collect::<Vec<_>>()
                        .join("\n\n");

                    embed = embed.field(
                        format!(
                            "Team 2 {}",
                            if team_2[0].mp_victory == 1 { "🏆" } else { "" }
                        ),
                        team_2_stats,
                        false,
                    );
                }

                msg.channel_id
                    .send_message(&ctx.http, CreateMessage::new().embed(embed))
                    .await?;
            }

            Err(e) => {
                let resp = format!("Error fetching stats: {}", e);
                msg.channel_id.say(&ctx.http, resp).await?;
            }
        }
        Ok(())
    }
}
