use async_trait::async_trait;
use krunker_rs::Client as KrunkerClient;
use serenity::all::{CreateEmbed, CreateMessage};
use serenity::model::channel::Message;
use serenity::prelude::*;
use sqlx::SqlitePool;
use std::sync::Arc;

use super::{all_commands, CommandMetadata, KrunkerCommand};

pub struct Help;

#[async_trait]
impl KrunkerCommand for Help {
    fn metadata(&self) -> CommandMetadata {
        CommandMetadata {
            name: "help",
            description: "Show this help message",
            usage: "&help",
            aliases: &["h"],
        }
    }

    #[allow(unused_variables)]
    async fn execute(
        &self,
        ctx: &Context,
        msg: &Message,
        krunker_api: &Arc<KrunkerClient>,
        args: Vec<&str>,
        _pool: &SqlitePool,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut embed = CreateEmbed::new()
            .title("Krunker Bot Help")
            .description("Available commands (Prefix: `&`)")
            .color(0x3498db)
            .footer(serenity::all::CreateEmbedFooter::new("Krunker RS Bot"));

        for cmd in all_commands() {
            let meta = cmd.metadata();
            embed = embed.field(
                format!("`{}`", meta.usage),
                meta.description,
                false,
            );
        }

        msg.channel_id
            .send_message(&ctx.http, CreateMessage::new().embed(embed))
            .await?;
            
        Ok(())
    }
}
