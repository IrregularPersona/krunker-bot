use crate::bot::commands::{Context, Error};
use chrono::TimeDelta;
use krunker_rs::MatchParticipant;
use serenity::all::CreateEmbed;
use poise::CreateReply;

/// Get detailed statistics for a specific match ID
#[poise::command(slash_command, prefix_command, aliases("sm"))]
pub async fn specific_match(
    ctx: Context<'_>,
    #[description = "The match ID to look up"] match_id: i64,
) -> Result<(), Error> {
    let krunker_api = &ctx.data().krunker_api;

    match krunker_api.get_match(match_id).await {
        Ok(data) => {
            let participants = match data.match_participants {
                Some(ref p) if !p.is_empty() => p,
                _ => {
                    ctx.say("No participants found for this match.").await?;
                    return Ok(());
                }
            };

            let dur = TimeDelta::milliseconds(data.match_duration as i64);
            let mins = dur.num_minutes();
            let secs = dur.num_seconds() % 60;

            let mut embed = CreateEmbed::new()
                .title(format!("Match Details - ID: {}", data.match_id))
                .field("Map", data.match_map.to_string(), true)
                .field("Duration", format!("{}m {}s", mins, secs), true)
                .field("Date", &data.match_date, true)
                .color(0x00ff00);

            let mut team_1: Vec<&MatchParticipant> = Vec::new();
            let mut team_2: Vec<&MatchParticipant> = Vec::new();

            for participant in participants {
                if participant.mp_team == 1 {
                    team_1.push(participant);
                } else {
                    team_2.push(participant);
                }
            }

            let format_player = |p: &MatchParticipant| -> String {
                let kda = format!("{}/{}/{}", p.mp_kills, p.mp_deaths, p.mp_assists);
                let result = if p.mp_victory == 1 { "🏆" } else { "" };
                format!(
                    "**{}** {}\nK/D/A: {} | Score: {}\nDamage: {} | Obj: {}",
                    p.mp_player_name,
                    result,
                    kda,
                    p.mp_score,
                    p.mp_damage_done,
                    p.mp_objective_score
                )
            };

            if !team_1.is_empty() {
                let team_1_stats = team_1
                    .iter()
                    .map(|p| format_player(p))
                    .collect::<Vec<_>>()
                    .join("\n\n");

                embed = embed.field(
                    format!(
                        "Team 1 {}",
                        if team_1[0].mp_victory == 1 { "🏆" } else { "" }
                    ),
                    team_1_stats,
                    false,
                );
            }

            if !team_2.is_empty() {
                let team_2_stats = team_2
                    .iter()
                    .map(|p| format_player(p))
                    .collect::<Vec<_>>()
                    .join("\n\n");

                embed = embed.field(
                    format!(
                        "Team 2 {}",
                        if team_2[0].mp_victory == 1 { "🏆" } else { "" }
                    ),
                    team_2_stats,
                    false,
                );
            }

            ctx.send(CreateReply::default().embed(embed)).await?;
        }
        Err(e) => {
            ctx.say(format!("Error fetching match stats: {}", e)).await?;
        }
    }
    Ok(())
}
