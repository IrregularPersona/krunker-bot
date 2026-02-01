use krunker_rs::Client as KrunkerClient;
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
pub mod unlink;

// Poise Data struct
pub struct Data {
    pub krunker_api: Arc<KrunkerClient>,
    pub pool: SqlitePool,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

pub fn all_commands() -> Vec<poise::Command<Data, Error>> {
    vec![
        ping::ping(),
        stats::stats(),
        ranked_stats::ranked_stats(),
        ranked_list::ranked_list(),
        specific_match::specific_match(),
        help::help(),
        link::link(),
        verify::verify(),
        unlink::unlink(),
    ]
}
