use crate::bot::commands::{Context, Error};
use serenity::all::CreateEmbed;
use poise::CreateReply;

/// List match IDs for the last N ranked matches
#[poise::command(slash_command, prefix_command, aliases("rl"))]
pub async fn ranked_list(
    ctx: Context<'_>,
    #[description = "The Krunker username to look up"] username: String,
    #[description = "Number of match IDs to show"] count: Option<usize>,
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
                .title(format!("Recent Ranked Match IDs - {}", username))
                .color(0x0000ff);

            for (i, pmatch) in matches.iter().take(count).enumerate() {
                embed = embed.field(
                    format!("Match #{}", i + 1),
                    pmatch.pm_match_id.to_string(),
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
