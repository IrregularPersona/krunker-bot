use crate::bot::commands::{Context, Error};
use serenity::all::CreateEmbed;
use poise::CreateReply;

/// Show general player statistics (K/D, Level, KR)
#[poise::command(slash_command, prefix_command)]
pub async fn stats(
    ctx: Context<'_>,
    #[description = "The Krunker username to look up"] username: String,
) -> Result<(), Error> {
    let krunker_api = &ctx.data().krunker_api;

    match krunker_api.get_player(&username).await {
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

            ctx.send(CreateReply::default().embed(embed)).await?;
        }
        Err(e) => {
            ctx.say(format!("Error fetching stats for **{}**: {}", username, e))
                .await?;
        }
    }
    Ok(())
}
