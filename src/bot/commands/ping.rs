use crate::bot::commands::{Context, Error};

/// Check if the bot is responsive
#[poise::command(slash_command, prefix_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("ping back").await?;
    Ok(())
}
