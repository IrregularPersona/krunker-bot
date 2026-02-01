use async_trait::async_trait;
use krunker_rs::Client as KrunkerClient;
use serenity::model::channel::Message;
use serenity::prelude::*;
use sqlx::SqlitePool;
use std::sync::Arc;

pub mod ping;
pub mod stats;
pub mod ranked_stats;
pub mod ranked_list;
pub mod specific_match;
pub mod help;
pub mod link;
pub mod verify;

pub struct CommandMetadata {
    pub name: &'static str,
    pub description: &'static str,
    pub usage: &'static str,
    pub aliases: &'static [&'static str],
}

#[async_trait]
pub trait KrunkerCommand: Send + Sync {
    fn metadata(&self) -> CommandMetadata;

    async fn execute(
        &self,
        ctx: &Context,
        msg: &Message,
        krunker_api: &Arc<KrunkerClient>,
        args: Vec<&str>,
        pool: &SqlitePool,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

pub fn all_commands() -> Vec<Arc<dyn KrunkerCommand>> {
    vec![
        Arc::new(ping::Ping),
        Arc::new(stats::Stats),
        Arc::new(ranked_stats::RankedStats),
        Arc::new(ranked_list::RankedList),
        Arc::new(specific_match::SpecificMatch),
        Arc::new(help::Help),
        Arc::new(link::Link),
        Arc::new(verify::Verify),
    ]
}
