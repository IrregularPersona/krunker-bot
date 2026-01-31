use crate::database::queries;
use chrono::Utc;
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

    // TODO: Fetch Krunker social posts and check for code
    // For now, we'll return a placeholder
    // This is where you'll call the Krunker API:
    // let posts = krunker_api::get_player_posts(&verification.krunker_username).await?;
    // let found = posts.iter().any(|post| post.text.contains(&verification.code));

    // Placeholder - always return NotFound for now
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
    // TODO: Fetch country from Krunker API
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
