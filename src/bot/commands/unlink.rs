use crate::bot::commands::{Context, Error};
use crate::database::queries;
use poise::CreateReply;

/// Unlink your Krunker account from your Discord account
#[poise::command(slash_command, prefix_command, ephemeral = true)]
pub async fn unlink(ctx: Context<'_>) -> Result<(), Error> {
    let pool = &ctx.data().pool;
    let discord_id = ctx.author().id.to_string();

    if !queries::user_exists(pool, &discord_id).await? {
        ctx.send(
            CreateReply::default()
                .content("You are not linked to any Krunker account.")
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    queries::delete_user(pool, &discord_id).await?;

    ctx.send(
        CreateReply::default()
            .content("✅ Successfully unlinked your account. You can now link a new one with `/link <username>`.")
            .ephemeral(true),
    )
    .await?;

    Ok(())
}
