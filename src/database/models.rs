#[derive(Debug, Clone, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub discord_id: String,
    pub country: Option<String>,
    pub day_created: i64,
}
