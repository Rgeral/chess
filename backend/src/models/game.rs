use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Represents a chess game between user and Stockfish
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub id: String,
    pub user_id: String,
    pub difficulty: i32,
    pub fen: String,
    pub moves: String,
    pub status: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_seconds: Option<i32>,
}