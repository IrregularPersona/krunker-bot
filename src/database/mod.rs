pub mod models;
pub mod queries;

use sqlx::{Result, SqlitePool};

pub async fn init_db() -> Result<SqlitePool> {
    let pool = SqlitePool::connect("sqlite:bot.db").await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
