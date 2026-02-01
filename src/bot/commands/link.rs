use crate::bot::commands::{Context, Error};
use poise::CreateReply;

/// Link your Krunker account to your Discord account
#[poise::command(slash_command, prefix_command, ephemeral = true)]
pub async fn link(
    ctx: Context<'_>,
    #[description = "Your Krunker username"] username: String,
) -> Result<(), Error> {
    let pool = &ctx.data().pool;
    let discord_id = ctx.author().id.to_string();

    match crate::verification::flow::start_verification(pool, &discord_id, &username).await {
        Ok(code) => {
            let response = format!(
                "Verification started for **{}**!\n\n\
                Please post the following code to your Krunker.io social profile:\n\
                `{}`\n\n\
                After posting, run `/verify` to complete the link.",
                username, code
            );
            ctx.send(CreateReply::default().content(response).ephemeral(true))
                .await?;
        }
        Err(e) => {
            ctx.send(
                CreateReply::default()
                    .content(format!("Error: {}", e))
                    .ephemeral(true),
            )
            .await?;
        }
    }

    Ok(())
}
