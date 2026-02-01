use crate::database::queries;
use chrono::Utc;
use krunker_rs::Client as KrunkerClient;
use rand::distr::{Alphanumeric, SampleString};
use sqlx::SqlitePool;

const VERIFICATION_EXPIRY_SECONDS: i64 = 120; // 2mins?

fn generate_code() -> String {
    let random_string = Alphanumeric.sample_string(&mut rand::rng(), 8);

    format!("VERIFY-{}", random_string.to_uppercase())
}

pub async fn start_verification(
    pool: &SqlitePool,
    discord_id: &str,
    krunker_username: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    if queries::user_exists(pool, discord_id).await? {
        return Err("You have already linked to a Krunker account. Use /unlink first.".into());
    }

    let code = generate_code();

    let now = Utc::now().timestamp();
    let expires_at = now + VERIFICATION_EXPIRY_SECONDS;

    sqlx::query!("DELETE FROM verifications WHERE discord_id = ?", discord_id)
        .execute(pool)
        .await?;

    queries::create_verification(pool, discord_id, krunker_username, &code, expires_at).await?;

    Ok(code)
}

pub async fn check_verification(
    pool: &SqlitePool,
    krunker_api: &KrunkerClient,
    discord_id: &str,
) -> Result<VerificationResult, Box<dyn std::error::Error>> {
    let expr = Utc::now().timestamp();
    let verification = sqlx::query_as::<_, crate::database::models::Verification>(
        "SELECT id, discord_id, krunker_username, code, created_at, expires_at, attempts
         FROM verifications 
         WHERE discord_id = ? AND expires_at > ?",
    )
    .bind(discord_id)
    .bind(expr)
    .fetch_optional(pool)
    .await?;

    let verification = match verification {
        Some(v) => v,
        None => return Ok(VerificationResult::NoVerification),
    };

    // Fetch Krunker social posts and check for code
    let response = krunker_api
        .get_player_posts(&verification.krunker_username, Some(1))
        .await?;

    // Check if any post contains the verification code
    let found = if let Some(posts) = response.posts_posts {
        posts
            .iter()
            .any(|post| post.post_text.contains(&verification.code))
    } else {
        false
    };

    if found {
        return Ok(VerificationResult::Success {
            krunker_username: verification.krunker_username,
        });
    }

    sqlx::query!(
        "UPDATE verifications SET attempts = attempts + 1 WHERE discord_id = ?",
        discord_id
    )
    .execute(pool)
    .await?;

    let new_attempts = verification.attempts + 1;

    if new_attempts >= 5 {
        sqlx::query!("DELETE FROM verifications WHERE discord_id = ?", discord_id)
            .execute(pool)
            .await?;

        return Err("Too many verification attempts (5). Please start over with /link.".into());
    }

    Ok(VerificationResult::NotFound {
        code: verification.code,
        krunker_username: verification.krunker_username,
        attempts: verification.attempts,
    })
}

/// Complete the verification and link the account
pub async fn complete_verification(
    pool: &SqlitePool,
    discord_id: &str,
    krunker_username: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let existing = queries::get_user_by_username(pool, krunker_username).await?;
    if let Some(res) = existing {
        if res.discord_id != discord_id {
            return Err(
                "This Krunker username is already linked to another Discord account.".into(),
            );
        }
    }

    let country = None;

    // Create user in database
    queries::create_user(pool, krunker_username, discord_id, country).await?;

    sqlx::query!("DELETE FROM verifications WHERE discord_id = ?", discord_id)
        .execute(pool)
        .await?;

    Ok(())
}

/// Result for a verification check
#[derive(Debug)]
pub enum VerificationResult {
    NoVerification,
    NotFound {
        code: String,
        krunker_username: String,
        attempts: i32,
    },
    Success {
        krunker_username: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_start_verification_success() {
        let pool = setup_test_db().await;
        let discord_id = "12345";
        let krunker_username = "Player1";

        let result = start_verification(&pool, discord_id, krunker_username).await;
        assert!(result.is_ok());

        let code = result.unwrap();
        assert!(code.starts_with("VERIFY-"));
        assert_eq!(code.len(), 15); // "VERIFY-" (7) + 8 chars
    }

    #[tokio::test]
    async fn test_start_verification_already_linked() {
        let pool = setup_test_db().await;
        let discord_id = "12345";

        // Pre-create user
        queries::create_user(&pool, "ExistingPlayer", discord_id, None)
            .await
            .unwrap();

        let result = start_verification(&pool, discord_id, "NewPlayer").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already linked"));
    }

    #[tokio::test]
    async fn test_complete_verification_already_exists() {
        let pool = setup_test_db().await;
        let discord_id = "12345";

        // Pre-create user
        queries::create_user(&pool, "Player1", discord_id, None)
            .await
            .unwrap();

        // Try to complete verification for same discord_id
        let result = complete_verification(&pool, discord_id, "Player2").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_ishaq_ayubi_verification_pull() {
        let pool = setup_test_db().await;

        // Pull API key from .env (KRUNKER_API)
        dotenvy::dotenv().ok();
        let api_key = std::env::var("KRUNKER_API").unwrap_or_else(|_| "DUMMY_KEY".to_string());
        let krunker_api = KrunkerClient::new(api_key).unwrap();

        let discord_id = "test_discord_user";
        let krunker_username = "IshaqAyubi";
        let code = "VERIFYB2C1A3";

        // Create the verification record manually for this test
        // Using Pepsi's account and just a random string for the verification
        let now = Utc::now().timestamp();
        queries::create_verification(&pool, discord_id, krunker_username, code, now + 600)
            .await
            .unwrap();

        println!(
            "Pulling social posts for: {} with code: {}...",
            krunker_username, code
        );

        // Pull posts and check verification
        match check_verification(&pool, &krunker_api, discord_id).await {
            Ok(result) => match result {
                VerificationResult::Success { krunker_username } => {
                    println!("Success! Found verification for {}", krunker_username);
                }
                VerificationResult::NotFound { code, .. } => {
                    println!(
                        "Verification code {} not found in posts for {}",
                        code, krunker_username
                    );
                }
                _ => println!("Unexpected result: {:?}", result),
            },
            Err(e) => {
                if e.to_string().contains("403") || e.to_string().contains("Not allowed") {
                    println!(
                        "Observation: API call failed with 403 (Invalid/Unauthorized API Key). This is expected if the key in .env is invalid."
                    );
                } else {
                    println!("Observation: Verification check failed with error: {}", e);
                }
            }
        }
    }
}
