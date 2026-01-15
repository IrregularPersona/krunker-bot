use std::sync::Arc;

use krunker_rs::Client as KrunkerClient;
use serenity::all::{CreateEmbed, CreateMessage};
// use serenity::all::Command;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;

struct Handler {
    krunker_api: Arc<KrunkerClient>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        if !msg.content.starts_with("&") {
            return;
        }

        let content = &msg.content[1..];
        let mut parts = content.split_whitespace();
        let command = parts.next().unwrap_or("");

        tracing::info!(
            user = %msg.author.name,
            user_id = %msg.author.id,
            command = %msg.content,
            "Command received"
        );

        match command {
            "ping" => {
                if let Err(why) = msg.channel_id.say(&ctx.http, "ping back").await {
                    eprintln!("Error sending message: {why:?}");
                }
            }

            "stats" => {
                let username = parts.next().unwrap_or("");

                if username.is_empty() {
                    if let Err(why) = msg
                        .channel_id
                        .say(&ctx.http, "Usage: &stats <username>")
                        .await
                    {
                        eprintln!("Error sending message: {why:?}");
                    }
                    return;
                }

                match self.krunker_api.get_player(username).await {
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

            "rankedstats" => {
                let username = parts.next().unwrap_or("");
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

                match self
                    .krunker_api
                    .get_player_matches(username, None, None)
                    .await
                {
                    Ok(data) => {
                        if data.pmr_matches.is_empty() {
                            if let Err(why) = msg
                                .channel_id
                                .say(&ctx.http, "No recent ranked data found!")
                                .await
                            {
                                eprintln!("Error sending message: {why:?}");
                            }
                            return;
                        }

                        let mut embed = CreateEmbed::new()
                            .title(format!("Recent Ranked Matches - {}", username))
                            .color(0x00ff00);

                        for (i, pmatch) in data.pmr_matches.iter().take(5).enumerate() {
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
                                format!("Match #{} - {}", i + 1, pmatch.pm_date),
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

            _ => {
                if let Err(why) = msg.channel_id.say(&ctx.http, "Not a valid command").await {
                    eprintln!("Error sending message: {why:?}");
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // env vars
    let krunker_key = std::env::var("KRUNKER_KEY")?;
    let discord_token = std::env::var("DISCORD_TOKEN2")?;

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // println!("discord token: {}", discord_token);

    let krunker_api = Arc::new(KrunkerClient::new(krunker_key)?);

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&discord_token, intents)
        .event_handler(Handler { krunker_api })
        .await
        .expect("Failure to create client");

    if let Err(why) = client.start().await {
        eprintln!("Client error: {why:?}");
    }

    Ok(())
}
