use serde::{Deserialize, Serialize};

pub const REACTION_EMOJI: &str = "⭐";
pub const APPROVED_EMOJI: &str = "🌠";

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd)]
pub enum Priority {
    Low,
    Medium,
    High,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PriorityChannel {
    pub id: u64,
    pub priority: Option<Priority>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub channels: Vec<PriorityChannel>,
    pub discord_token: String,
    pub discord_channel: u64,
    pub discord_server: u64,
    pub threshold: u64,
    pub reply: bool,
    pub replies: Vec<String>,
    pub enable_channel_whitelist: Option<bool>,
    pub persistence_file: Option<String>,
}
