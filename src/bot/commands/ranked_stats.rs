use crate::bot::commands::{Context, Error};
use serenity::all::CreateEmbed;
use poise::CreateReply;

/// Show detailed stats for the last N ranked matches
#[poise::command(slash_command, prefix_command)]
pub async fn ranked_stats(
    ctx: Context<'_>,
    #[description = "The Krunker username to look up"] username: String,
    #[description = "Number of matches to show"] count: Option<usize>,
) -> Result<(), Error> {
    let krunker_api = &ctx.data().krunker_api;
    let count = count.unwrap_or(1);

    match krunker_api.get_player_matches(&username, None, None).await {
        Ok(data) => {
            let matches = data.pmr_matches.unwrap_or_default();
            if matches.is_empty() {
                ctx.say("No recent ranked data found!").await?;
                return Ok(());
            }

            let mut embed = CreateEmbed::new()
                .title(format!("Recent Ranked Matches - {}", username))
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

            ctx.send(CreateReply::default().embed(embed)).await?;
        }
        Err(e) => {
            ctx.say(format!("Error fetching stats: {}", e)).await?;
        }
    }
    Ok(())
}
