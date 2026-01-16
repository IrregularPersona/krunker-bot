use async_trait::async_trait;
use serenity::all::{CreateAttachment, CreateEmbed, CreateMessage};
use serenity::model::channel::Message;
use serenity::prelude::*;

use krunker_rs::Client as KrunkerClient;
use std::sync::Arc;

use crate::pngrender::{get_png, StatInputs};

#[async_trait]
pub trait KrunkerCommand {
    async fn execute(
        &self,
        ctx: &Context,
        msg: &Message,
        krunker_api: &Arc<KrunkerClient>,
        args: Vec<&str>,
    );
}

pub struct Ping;

#[async_trait]
impl KrunkerCommand for Ping {
    #[allow(unused_variables)]
    async fn execute(
        &self,
        ctx: &Context,
        msg: &Message,
        krunker_api: &Arc<KrunkerClient>,
        args: Vec<&str>,
    ) {
        if let Err(why) = msg.channel_id.say(&ctx.http, "ping back").await {
            tracing::error!("Error sending message: {why:?}");
        }
    }
}

pub struct Stats;

#[async_trait]
impl KrunkerCommand for Stats {
    #[allow(unused_variables)]
    async fn execute(
        &self,
        ctx: &Context,
        msg: &Message,
        krunker_api: &Arc<KrunkerClient>,
        args: Vec<&str>,
    ) {
        let username = args.get(0).unwrap_or(&"");

        if username.is_empty() {
            if let Err(why) = msg
                .channel_id
                .say(&ctx.http, "Usage: &stats <username>")
                .await
            {
                tracing::error!("Error sending message: {why:?}");
            }
            return;
        }

        match krunker_api.get_player(username).await {
            Ok(player) => {
                // Generate PNG using the pngrender module
                let png_data = match get_png(StatInputs::Profile(player.clone())) {
                    Some(data) => data,
                    None => {
                        if let Err(_) = msg
                            .channel_id
                            .say(&ctx.http, "Failed to generate player stats image")
                            .await
                        {
                            tracing::error!("Failed to generate PNG for player {}", username);
                        }
                        return;
                    }
                };

                // Create attachment from PNG data
                let attachment = CreateAttachment::bytes(png_data, format!("{}_stats.png", player.player_name));

                // Send the PNG as an attachment
                if let Err(why) = msg
                    .channel_id
                    .send_message(&ctx.http, CreateMessage::new().add_file(attachment))
                    .await
                {
                    tracing::error!("Error sending message: {why:?}");
                }
            }

            Err(e) => {
                let response = format!("Error fetching stats: {}", e);
                if let Err(why) = msg.channel_id.say(&ctx.http, response).await {
                    tracing::error!("Error sending message: {why:?}");
                }
            }
        }
    }
}

pub struct RankedStats;

#[async_trait]
impl KrunkerCommand for RankedStats {
    async fn execute(
        &self,
        ctx: &Context,
        msg: &Message,
        krunker_api: &Arc<KrunkerClient>,
        args: Vec<&str>,
    ) {
        let username = args.get(0).unwrap_or(&"");
        let count_str = args.get(1).unwrap_or(&"");

        if username.is_empty() {
            if let Err(why) = msg
                .channel_id
                .say(&ctx.http, "Usage: &rankedstats <username>")
                .await
            {
                eprintln!("Error sending message: {why:?}");
            }
            return;
        }

        let mut count: i32 = 1;
        if !count_str.is_empty() {
            count = count_str.parse::<i32>().expect("Failure to parse");
        }

        match krunker_api.get_player_matches(username, None, None).await {
            Ok(data) => {
                let matches = data.pmr_matches.unwrap_or_default();
                if matches.is_empty() {
                    if let Err(_) = msg
                        .channel_id
                        .say(&ctx.http, "No recent ranked data found!")
                        .await
                    {
                        tracing::info!("Ranked data for {} is empty.", username);
                    }
                    return;
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

                if let Err(why) = msg
                    .channel_id
                    .send_message(&ctx.http, CreateMessage::new().embed(embed))
                    .await
                {
                    eprintln!("Error sending message: {why:?}");
                }
            }
            Err(e) => {
                let response = format!("Error fetching stats: {}", e);
                if let Err(why) = msg.channel_id.say(&ctx.http, response).await {
                    eprintln!("Error sending message: {why:?}");
                }
            }
        }
    }
}

pub struct RankedList;

#[async_trait]
impl KrunkerCommand for RankedList {
    async fn execute(
        &self,
        ctx: &Context,
        msg: &Message,
        krunker_api: &Arc<KrunkerClient>,
        args: Vec<&str>,
    ) {
        let username = args.get(0).unwrap_or(&"");
        if username.is_empty() {
            if let Err(why) = msg
                .channel_id
                .say(&ctx.http, "Usage: &rl <username> <optional: int>")
                .await
            {
                eprintln!("Error sending message: {why:?}");
            }
            return;
        }

        let count_str = args.get(1).unwrap_or(&"");
        let mut count: i32 = 1;
        if !count_str.is_empty() {
            count = count_str.parse::<i32>().expect("Failure to parse");
        }

        let pdata = krunker_api.get_player_matches(username, None, None).await;

        match pdata {
            Ok(data) => {
                let matches = data.pmr_matches.unwrap_or_default();
                if matches.is_empty() {
                    if let Err(_) = msg
                        .channel_id
                        .say(&ctx.http, "No recent ranked data found!")
                        .await
                    {
                        tracing::info!("Ranked data for {} is empty.", username);
                    }
                    return;
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

                if let Err(e) = msg
                    .channel_id
                    .send_message(&ctx.http, CreateMessage::new().embed(embed))
                    .await
                {
                    tracing::error!("Error sending message: {e:?}");
                }
            }
            Err(e) => {
                let response = format!("Error fetching stats: {}", e);
                if let Err(why) = msg.channel_id.say(&ctx.http, response).await {
                    tracing::error!("Error sending message: {why:?}");
                }
            }
        }
    }
}

pub struct SpecificMatch;

#[async_trait]
impl KrunkerCommand for SpecificMatch {
    async fn execute(
        &self,
        ctx: &Context,
        msg: &Message,
        krunker_api: &Arc<KrunkerClient>,
        args: Vec<&str>,
    ) {
        let id_str = args.get(0).unwrap_or(&"");
        let match_id = id_str.parse::<i64>().expect("Failure to parse");

        if match_id <= 0 {
            if let Err(_) = msg
                .channel_id
                .say(&ctx.http, "Usage: &sm <int: match_id>")
                .await
            {
                tracing::info!("Invalid ID");
            }
            return;
        }

        let pdata = krunker_api.get_match(match_id).await;

        match pdata {
            Ok(data) => {
                let participants = match data.match_participants {
                    Some(ref p) if !p.is_empty() => p,
                    _ => {
                        if let Err(_) = msg.channel_id.say(&ctx.http, "Report to Glen????").await {
                            tracing::error!("No participants??");
                        }
                        return;
                    }
                };

                // Generate PNG using the pngrender module
                let png_data = match get_png(StatInputs::Match(participants.clone())) {
                    Some(data) => data,
                    None => {
                        if let Err(_) = msg
                            .channel_id
                            .say(&ctx.http, "Failed to generate match image")
                            .await
                        {
                            tracing::error!("Failed to generate PNG for match {}", match_id);
                        }
                        return;
                    }
                };

                // Create attachment from PNG data
                let attachment = CreateAttachment::bytes(png_data, format!("match_{}.png", data.match_id));

                // Send the PNG as an attachment
                if let Err(why) = msg
                    .channel_id
                    .send_message(&ctx.http, CreateMessage::new().add_file(attachment))
                    .await
                {
                    tracing::error!("Error sending message: {why:?}");
                }
            }

            Err(e) => {
                let resp = format!("Error fetching stats: {}", e);
                if let Err(why) = msg.channel_id.say(&ctx.http, resp).await {
                    tracing::error!("Error sending message: {why:?}");
                }
            }
        }
    }
}

pub struct Help;

#[async_trait]
impl KrunkerCommand for Help {
    #[allow(unused_variables)]
    async fn execute(
        &self,
        ctx: &Context,
        msg: &Message,
        krunker_api: &Arc<KrunkerClient>,
        args: Vec<&str>,
    ) {
        let embed = CreateEmbed::new()
            .title("Krunker Bot Help")
            .description("Available commands (Prefix: `&`)")
            .field(
                "`&p <username>`",
                "Show general player statistics (K/D, Level, KR).",
                false,
            )
            .field(
                "`&r <user> [count]`",
                "Show detailed stats for the last N ranked matches.",
                false,
            )
            .field(
                "`&rl <user> [count]`",
                "List match IDs for the last N ranked matches.",
                false,
            )
            .field(
                "`&sm <match_id>`",
                "Get detailed statistics for a specific match ID.",
                false,
            )
            .field("`&ping`", "Check if the bot is responsive.", false)
            .color(0x3498db) // random blue
            .footer(serenity::all::CreateEmbedFooter::new("Krunker RS Bot"));

        if let Err(why) = msg
            .channel_id
            .send_message(&ctx.http, CreateMessage::new().embed(embed))
            .await
        {
            tracing::error!("Error sending help message: {why:?}");
        }
    }
}
