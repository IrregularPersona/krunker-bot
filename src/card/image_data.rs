#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PersonalProfileData {
    pub player_name: String,
    pub clan: Option<String>,
    pub level: i32,
    pub level_xp: (i64, i64), // (current, max)
    pub verified: bool,
    pub followers: i32,
    pub following: i32,
    pub created_date: String,

    // Performance details
    pub kills: i32,
    pub deaths: i32,
    pub kdr: f64,
    pub accuracy: f64,
    pub kr: i32,
    pub time_played: String,
    pub nukes: i32,
    // Ranked stats (latest/best)
    // pub ranked: Option<RankedProfileData>,
}

// #[allow(dead_code)]
// #[derive(Debug, Clone)]
// pub struct RankedProfileData {
//     pub rank_name: String,
//     pub mmr: i32,
//     pub next_rank_mmr: i32,
//     pub kdr: f64,
//     pub win_rate: f64,
// }

// #[allow(dead_code)]
// #[derive(Debug, Clone)]
// pub struct ClanData {
//     pub name: String,
//     pub owner: String,
//     pub member_count: i32,
// }

// #[allow(dead_code)]
// #[derive(Debug, Clone)]
// pub struct RankedLeaderboardData {
//     pub entries: Vec<RankedProfileData>,
// }

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum CardData {
    PersonalProfileCard(PersonalProfileData),
    // RankedProfileCard(RankedProfileData),
    // ClanCard(ClanData),
    // RankedLeaderboardCard(RankedLeaderboardData),
}
