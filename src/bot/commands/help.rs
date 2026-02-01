use crate::bot::commands::{Context, Error};

/// Show help documentation
#[poise::command(slash_command, prefix_command, aliases("h"))]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help for"] command: Option<String>,
) -> Result<(), Error> {
    poise::samples::help(
        ctx,
        command.as_deref(),
        poise::samples::HelpConfiguration {
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}
