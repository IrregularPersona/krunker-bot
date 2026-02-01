use crate::bot::commands::{Context, Error};
use crate::database::queries;
use serenity::all::CreateEmbed;
use poise::CreateReply;

/// Show general player statistics (K/D, Level, KR)
#[poise::command(slash_command, prefix_command)]
pub async fn stats(
    ctx: Context<'_>,
    #[description = "The Krunker username to look up (optional if linked)"] username: Option<String>,
) -> Result<(), Error> {
    let krunker_api = &ctx.data().krunker_api;
    let pool = &ctx.data().pool;

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

    match krunker_api.get_player(&target_username).await {
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
            ctx.say(format!("Error fetching stats for **{}**: {}", target_username, e))
                .await?;
        }
    }
    Ok(())
}
