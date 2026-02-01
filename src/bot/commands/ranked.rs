use crate::bot::commands::{Context, Error};
use crate::database::queries;
use serenity::all::CreateEmbed;
use poise::CreateReply;

#[derive(poise::ChoiceParameter)]
pub enum RankedMode {
    #[name = "stats"]
    Stats,
    #[name = "list"]
    List,
}

/// Show ranked match information (stats or match IDs)
#[poise::command(slash_command, prefix_command, aliases("r", "rl"))]
pub async fn ranked(
    ctx: Context<'_>,
    #[description = "The Krunker username to look up (optional if linked)"] username: Option<String>,
    #[description = "Number of matches to show"] count: Option<usize>,
    #[description = "Show detailed stats or just a list of IDs"] mode: Option<RankedMode>,
) -> Result<(), Error> {
    let krunker_api = &ctx.data().krunker_api;
    let pool = &ctx.data().pool;
    let count = count.unwrap_or(1);
    let mode = mode.unwrap_or(RankedMode::Stats);

    let target_username = if let Some(u) = username {
        u
    } else {
        let discord_id = ctx.author().id.to_string();
        if let Some(user) = queries::get_user_by_discord_id(pool, &discord_id).await? {
            user.username
        } else {
            ctx.say("Please provide a username or link your account first with `/link`.")
                .await?;
            return Ok(());
        }
    };

    match krunker_api.get_player_matches(&target_username, None, None).await {
        Ok(data) => {
            let matches = data.pmr_matches.unwrap_or_default();
            if matches.is_empty() {
                ctx.say(format!("No recent ranked data found for **{}**!", target_username)).await?;
                return Ok(());
            }

            let mut embed = CreateEmbed::new();
            
            match mode {
                RankedMode::Stats => {
                    embed = embed.title(format!("Recent Ranked Matches - {}", target_username))
                        .color(0x00ff00);

                    for pmatch in matches.iter().take(count) {
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
                },
                RankedMode::List => {
                    embed = embed.title(format!("Recent Ranked Match IDs - {}", target_username))
                        .color(0x0000ff);

                    for (i, pmatch) in matches.iter().take(count).enumerate() {
                        embed = embed.field(
                            format!("Match #{}", i + 1),
                            pmatch.pm_match_id.to_string(),
                            false,
                        );
                    }
                }
            }

            ctx.send(CreateReply::default().embed(embed)).await?;
        }
        Err(e) => {
            ctx.say(format!("Error fetching ranked stats for **{}**: {}", target_username, e)).await?;
        }
    }
    Ok(())
}
