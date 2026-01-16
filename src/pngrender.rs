use krunker_rs::{MatchParticipant, Player};
use typst_png::compile;

const FONT_ROBOTO: &[u8] = include_bytes!("../resource/font/RobotoMono-Medium.ttf");

const TEMPLATE: &str = r#"#let n = 2.5fr
#set page(width: 75cm, height: 20cm, margin: 1cm)
#set text(size: 25pt)
#columns(2, gutter: 20pt, {
  align(center, block(radius: 8pt, stroke: 1pt + blue, clip: true,
    table(
      columns: (n, 7cm, n, n, n, n, n),
      rows: (n, n, n, n, n),
      align: center + horizon,
      table.header(table.cell(colspan: 7)[*Team 1*]),
      [-], [*Player*], [*K/D/A*], [*Score*], [*Damage*], [*Obj*], [*Acc (%)*],
{{TEAM1_ROWS}}
    )
  ))
  
  colbreak()
  
  align(center, block(radius: 8pt, stroke: 1pt + red, clip: true,
    table(
      columns: (n, 7cm, n, n, n, n, n),
      rows: (n, n, n, n, n),
      align: center + horizon,
      table.header(table.cell(colspan: 7)[*Team 2*]),
      [-], [*Player*], [*K/D/A*], [*Score*], [*Damage*], [*Obj*], [*Acc (%)*],
{{TEAM2_ROWS}}
    )
  ))
})
"#;

const PROFILE_TEMPLATE: &str = r#"#let n = 2.5fr
#set page(width: 15cm, height: 5cm, margin: 1cm)
#set text(size: 15pt)
#figure(
  table(
    columns: (n, n, n, n),
    rows: (n, n),
    align: center+horizon,
    stroke: (x: none),
    table.header(
      table.cell(colspan: 2)[{{USERNAME}}], table.cell(colspan: 2)[{{CLAN}}],
    ),
    [Level \ {{LEVEL}}], [KDR \ {{KDR}}], [KR \ {{KR}}], [GP \ {{GP}}],
  )
)
"#;

// do i need to pass some sort of config later?
pub enum StatInputs {
    Match(Vec<MatchParticipant>),
    Profile(Player),
}

fn build_player_row(rank: usize, participant: &MatchParticipant) -> String {
    format!(
        "      [{}], [{}], [{}/{}/{}], [{}], [{}], [{}], [{}%],",
        rank,
        participant.mp_player_name,
        participant.mp_kills,
        participant.mp_deaths,
        participant.mp_assists,
        participant.mp_score,
        participant.mp_damage_done,
        participant.mp_objective_score,
        participant.mp_accuracy
    )
}

fn generate_match_string(participants: Vec<MatchParticipant>) -> String {
    let team1: Vec<_> = participants.iter().filter(|p| p.mp_team == 1).collect();

    let team2: Vec<_> = participants.iter().filter(|p| p.mp_team != 1).collect();

    let team1_rows = team1
        .iter()
        .enumerate()
        .map(|(i, p)| build_player_row(i + 1, p))
        .collect::<Vec<_>>()
        .join("\n");

    let team2_rows = team2
        .iter()
        .enumerate()
        .map(|(i, p)| build_player_row(i + 1, p))
        .collect::<Vec<_>>()
        .join("\n");

    TEMPLATE
        .replace("{{TEAM1_ROWS}}", &team1_rows)
        .replace("{{TEAM2_ROWS}}", &team2_rows)
}

fn generate_profile_string(profile: Player) -> String {
    PROFILE_TEMPLATE
        .replace("{{USERNAME}}", &profile.player_name)
        .replace("{{CLAN}}", &profile.player_clan)
        .replace("{{LEVEL}}", &profile.player_level.to_string())
        .replace("{{KDR}}", &profile.player_kdr.to_string())
        .replace("{{KR}}", &profile.player_kr.to_string())
        .replace("{{GP}}", &profile.player_games.to_string())
}

fn generate_stats(input: StatInputs) -> String {
    match input {
        StatInputs::Match(participants) => generate_match_string(participants),
        StatInputs::Profile(profile) => generate_profile_string(profile),
    }
}

pub fn get_png(input: StatInputs) -> Option<Vec<u8>> {
    let template = generate_stats(input);
    let temp_vec = vec![FONT_ROBOTO.to_vec()];

    match compile(template, temp_vec) {
        Ok(png_result) => Some(png_result),
        Err(_) => None,
    }
}
