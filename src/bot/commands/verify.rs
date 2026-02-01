use crate::bot::commands::{Context, Error};
use poise::CreateReply;

/// Verify your linked Krunker account
#[poise::command(slash_command, prefix_command, ephemeral = true)]
pub async fn verify(ctx: Context<'_>) -> Result<(), Error> {
    use crate::verification::flow::{check_verification, complete_verification, VerificationResult};

    let pool = &ctx.data().pool;
    let krunker_api = &ctx.data().krunker_api;
    let discord_id = ctx.author().id.to_string();

    match check_verification(pool, krunker_api, &discord_id).await {
        Ok(VerificationResult::Success { krunker_username }) => {
            complete_verification(pool, &discord_id, &krunker_username).await?;
            ctx.send(
                CreateReply::default()
                    .content(format!(
                        "✅ Successfully verified! Your Discord account is now linked to **{}**.",
                        krunker_username
                    ))
                    .ephemeral(true),
            )
            .await?;
        }
        Ok(VerificationResult::NotFound {
            code,
            krunker_username,
            attempts,
        }) => {
            let response = format!(
                "❌ Verification code not found for **{}**.\n\n\
                Make sure you've posted this exactly: `{}`\n\
                Attempts: {}/5",
                krunker_username, code, attempts
            );
            ctx.send(CreateReply::default().content(response).ephemeral(true))
                .await?;
        }
        Ok(VerificationResult::NoVerification) => {
            ctx.send(
                CreateReply::default()
                    .content("You don't have an active verification session. Use `/link <username>` first.")
                    .ephemeral(true),
            )
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
