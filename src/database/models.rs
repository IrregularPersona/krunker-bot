#[derive(Debug, Clone, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub discord_id: String,
    pub country: Option<String>,
    pub day_created: i64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Verification {
    pub id: i64,
    pub discord_id: String,
    pub krunker_username: String,
    pub code: String,
    pub created_at: i64,
    pub expires_at: i64,
    pub attempts: i32,
}
