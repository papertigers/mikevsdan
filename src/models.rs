use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct ApiData {
    pub data: Vec<PlayerData>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
#[serde(tag = "type")]
pub enum PlayerData {
    PlayerTeamSeasonStats {
        #[serde(rename = "attributes")]
        stats: Stats,
    },
    #[serde(other)]
    _Ignore,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Player {
    pub stats: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Stats {
    pub games_played: u32,
    pub goals: u32,
    pub assists: u32,
    pub penalty_minutes: u32,
    pub hat_tricks: u32,
    pub points: u32,
}
