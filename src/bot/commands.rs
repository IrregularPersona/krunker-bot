use async_trait::async_trait;
use chrono::TimeDelta;
use krunker_rs::Client as KrunkerClient;
use krunker_rs::MatchParticipant;
use serenity::all::{CreateEmbed, CreateMessage};
use serenity::model::channel::Message;
use serenity::prelude::*;
use sqlx::SqlitePool;
use std::sync::Arc;

pub struct CommandMetadata {
    pub name: &'static str,
    pub description: &'static str,
    pub usage: &'static str,
    pub aliases: &'static [&'static str],
}

#[async_trait]
pub trait KrunkerCommand: Send + Sync {
    fn metadata(&self) -> CommandMetadata;

    async fn execute(
        &self,
        ctx: &Context,
        msg: &Message,
        krunker_api: &Arc<KrunkerClient>,
        args: Vec<&str>,
        pool: &SqlitePool,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

pub fn all_commands() -> Vec<Arc<dyn KrunkerCommand>> {
    vec![
        Arc::new(Ping),
        Arc::new(Stats),
        Arc::new(RankedStats),
        Arc::new(RankedList),
        Arc::new(SpecificMatch),
        Arc::new(Help),
        Arc::new(Link),
        Arc::new(Verify),
    ]
}

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

pub struct Help;

#[async_trait]
impl KrunkerCommand for Help {
    fn metadata(&self) -> CommandMetadata {
        CommandMetadata {
            name: "help",
            description: "Show this help message",
            usage: "&help",
            aliases: &["h"],
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
        let mut embed = CreateEmbed::new()
            .title("Krunker Bot Help")
            .description("Available commands (Prefix: `&`)")
            .color(0x3498db)
            .footer(serenity::all::CreateEmbedFooter::new("Krunker RS Bot"));

        for cmd in all_commands() {
            let meta = cmd.metadata();
            embed = embed.field(
                format!("`{}`", meta.usage),
                meta.description,
                false,
            );
        }

        msg.channel_id
            .send_message(&ctx.http, CreateMessage::new().embed(embed))
            .await?;
            
        Ok(())
    }
}

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
        krunker_api: &Arc<KrunkerClient>,
        args: Vec<&str>,
        _pool: &SqlitePool,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        msg.channel_id.say(&ctx.http, "Account linking is not yet implemented.").await?;
        Ok(())
    }
}

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
        args: Vec<&str>,
        _pool: &SqlitePool,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        msg.channel_id.say(&ctx.http, "Verification is not yet implemented.").await?;
        Ok(())
    }
}
