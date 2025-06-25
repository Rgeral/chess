use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Represents a winning game record for leaderboards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Score {
    pub id: String,
    pub user_id: String,
    pub game_id: String,
    pub difficulty: i32,
    pub duration_seconds: i32,
    pub created_at: DateTime<Utc>,
}

/// Represents a user's personal best time for a specific difficulty
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBestTime {
    pub user_id: String,
    pub difficulty: i32,
    pub best_time_seconds: i32,
    pub achieved_at: DateTime<Utc>,
}