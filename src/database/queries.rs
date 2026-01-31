use super::models::User;
use sqlx::{Result, SqlitePool};

// user related queries
pub async fn create_user(
    pool: &SqlitePool,
    username: &str,
    discord_id: &str,
    country: Option<&str>,
) -> Result<i64> {
    let result = sqlx::query!(
        "INSERT INTO users (username, discord_id, country) VALUES (?, ?, ?)",
        username,
        discord_id,
        country
    )
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

pub async fn get_user_by_discord_id(pool: &SqlitePool, discord_id: &str) -> Result<Option<User>> {
    sqlx::query_as::<_, User>(
        "SELECT id, username, discord_id, country, day_created FROM users WHERE discord_id = ?",
    )
    .bind(discord_id)
    .fetch_optional(pool)
    .await
}

pub async fn get_user_by_username(pool: &SqlitePool, username: &str) -> Result<Option<User>> {
    sqlx::query_as::<_, User>(
        "SELECT id, username, discord_id, country, day_created FROM users WHERE username = ?",
    )
    .bind(username)
    .fetch_optional(pool)
    .await
}

pub async fn delete_user(pool: &SqlitePool, discord_id: &str) -> Result<()> {
    sqlx::query!("DELETE FROM users WHERE discord_id = ?", discord_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn user_exists(pool: &SqlitePool, discord_id: &str) -> Result<bool> {
    let result = sqlx::query!(
        "SELECT COUNT(*) as count FROM users WHERE discord_id = ?",
        discord_id
    )
    .fetch_one(pool)
    .await?;
    Ok(result.count > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    #[tokio::test]
    async fn test_create_and_get_user() {
        // Create in-memory database for testing
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        // Test: Create user
        let user_id = create_user(&pool, "TestPlayer", "123456789", Some("US"))
            .await
            .unwrap();

        assert!(user_id > 0);

        // Test: Get user by Discord ID
        let user = get_user_by_discord_id(&pool, "123456789")
            .await
            .unwrap()
            .expect("User should exist");

        assert_eq!(user.username, "TestPlayer");
        assert_eq!(user.discord_id, "123456789");
        assert_eq!(user.country, Some("US".to_string()));
    }

    #[tokio::test]
    async fn test_get_user_by_username() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        create_user(&pool, "Player123", "999", None).await.unwrap();

        let user = get_user_by_username(&pool, "Player123")
            .await
            .unwrap()
            .expect("User should exist");

        assert_eq!(user.discord_id, "999");
    }

    #[tokio::test]
    async fn test_user_exists() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        create_user(&pool, "Test", "111", None).await.unwrap();

        assert!(user_exists(&pool, "111").await.unwrap());
        assert!(!user_exists(&pool, "999").await.unwrap());
    }

    #[tokio::test]
    async fn test_delete_user() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        create_user(&pool, "ToDelete", "333", None).await.unwrap();
        delete_user(&pool, "333").await.unwrap();

        assert!(!user_exists(&pool, "333").await.unwrap());
    }
}
