use crate::database::models::Verification;

use super::models::User;
use sqlx::{Result, SqlitePool};

// ========= USER SECTION
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

// ========= USER SECTION OVER

// ========= VERIFICATION SECTION

pub async fn create_verification(
    pool: &SqlitePool,
    discord_id: &str,
    krunker_username: &str,
    code: &str,
    expires_at: i64,
) -> Result<i64> {
    let result =sqlx::query!(
        "INSERT INTO verifications (discord_id, krunker_username, code, expires_at) VALUES (?, ?, ?, ?)",
        discord_id,
        krunker_username,
        code,
        expires_at
    )
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

pub async fn get_verification_by_code(
    pool: &SqlitePool,
    code: &str,
) -> Result<Option<Verification>> {
    sqlx::query_as::<_, Verification>(
        "SELECT id, discord_id, krunker_username, code, created_at, expires_at, attempts
        FROM verifications
        WHERE code = ? AND expires_at > strftime('%s', 'now')",
    )
    .bind(code)
    .fetch_optional(pool)
    .await
}

pub async fn delete_verification(pool: &SqlitePool, code: &str) -> Result<()> {
    sqlx::query!("DELETE FROM verifications WHERE code = ?", code)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn cleanup_expired_verifications(pool: &SqlitePool) -> Result<()> {
    sqlx::query!("DELETE FROM verifications WHERE expires_at < strftime('%s', 'now')")
        .execute(pool)
        .await?;
    Ok(())
}

// ========= VERIFICATION SECTION OVER

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        pool
    }

    // User tests
    #[tokio::test]
    async fn test_create_and_get_user() {
        let pool = setup_test_db().await;

        let user_id = create_user(&pool, "TestPlayer", "123456789", Some("US"))
            .await
            .unwrap();

        assert!(user_id > 0);

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
        let pool = setup_test_db().await;

        create_user(&pool, "Player123", "999", None).await.unwrap();

        let user = get_user_by_username(&pool, "Player123")
            .await
            .unwrap()
            .expect("User should exist");

        assert_eq!(user.discord_id, "999");
    }

    #[tokio::test]
    async fn test_user_exists() {
        let pool = setup_test_db().await;

        create_user(&pool, "Test", "111", None).await.unwrap();

        assert!(user_exists(&pool, "111").await.unwrap());
        assert!(!user_exists(&pool, "999").await.unwrap());
    }

    #[tokio::test]
    async fn test_delete_user() {
        let pool = setup_test_db().await;

        create_user(&pool, "ToDelete", "333", None).await.unwrap();
        delete_user(&pool, "333").await.unwrap();

        assert!(!user_exists(&pool, "333").await.unwrap());
    }

    // Verification tests
    #[tokio::test]
    async fn test_create_and_get_verification() {
        let pool = setup_test_db().await;

        let now = chrono::Utc::now().timestamp();
        let expires = now + 600; // 10 minutes from now

        let verification_id = create_verification(
            &pool,
            "discord123",
            "KrunkerPlayer",
            "VERIFY-ABC123",
            expires,
        )
        .await
        .unwrap();

        assert!(verification_id > 0);

        let verification = get_verification_by_code(&pool, "VERIFY-ABC123")
            .await
            .unwrap()
            .expect("Verification should exist");

        assert_eq!(verification.discord_id, "discord123");
        assert_eq!(verification.krunker_username, "KrunkerPlayer");
        assert_eq!(verification.code, "VERIFY-ABC123");
        assert_eq!(verification.attempts, 0);
    }

    #[tokio::test]
    async fn test_expired_verification_not_returned() {
        let pool = setup_test_db().await;

        let now = chrono::Utc::now().timestamp();
        let expires = now - 100; // Expired 100 seconds ago

        create_verification(&pool, "discord456", "Player", "VERIFY-EXPIRED", expires)
            .await
            .unwrap();

        // Should not return expired verification
        let result = get_verification_by_code(&pool, "VERIFY-EXPIRED")
            .await
            .unwrap();

        assert!(
            result.is_none(),
            "Expired verification should not be returned"
        );
    }

    #[tokio::test]
    async fn test_delete_verification() {
        let pool = setup_test_db().await;

        let now = chrono::Utc::now().timestamp();
        let expires = now + 600;

        create_verification(&pool, "discord789", "Player", "VERIFY-DELETE", expires)
            .await
            .unwrap();

        delete_verification(&pool, "VERIFY-DELETE").await.unwrap();

        let result = get_verification_by_code(&pool, "VERIFY-DELETE")
            .await
            .unwrap();

        assert!(result.is_none(), "Deleted verification should not exist");
    }

    #[tokio::test]
    async fn test_cleanup_expired_verifications() {
        let pool = setup_test_db().await;

        let now = chrono::Utc::now().timestamp();

        // Create expired verification
        create_verification(&pool, "d1", "p1", "CODE1", now - 100)
            .await
            .unwrap();

        // Create valid verification
        create_verification(&pool, "d2", "p2", "CODE2", now + 600)
            .await
            .unwrap();

        // Cleanup expired
        cleanup_expired_verifications(&pool).await.unwrap();

        // Expired should be gone
        assert!(
            get_verification_by_code(&pool, "CODE1")
                .await
                .unwrap()
                .is_none()
        );

        // Valid should still exist
        assert!(
            get_verification_by_code(&pool, "CODE2")
                .await
                .unwrap()
                .is_some()
        );
    }

    #[tokio::test]
    async fn test_unique_verification_code() {
        let pool = setup_test_db().await;

        let now = chrono::Utc::now().timestamp();
        let expires = now + 600;

        // Create first verification
        create_verification(&pool, "d1", "p1", "SAME-CODE", expires)
            .await
            .unwrap();

        // Try to create duplicate code - should fail
        let result = create_verification(&pool, "d2", "p2", "SAME-CODE", expires).await;

        assert!(result.is_err(), "Duplicate code should fail");
    }
}
