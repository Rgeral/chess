use async_graphql::*;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};

/// User model representing a chess player
#[derive(Debug, Clone, FromRow, Serialize, Deserialize, SimpleObject)]
pub struct User {
    /// Unique identifier for the user
    pub id: String,
    /// Display name chosen by the user
    pub username: String,
    /// Total number of games played
    pub total_games: i32,
    /// Number of games won
    pub games_won: i32,
    /// When the user account was created
    pub created_at: DateTime<Utc>,
    /// Total time spent playing (in seconds)
    pub total_play_time_seconds: Option<i32>,
    /// Current winning streak
    pub current_streak: Option<i32>,
    /// Best winning streak achieved
    pub best_streak: Option<i32>,
    /// Estimated ELO rating based on performance
    pub estimated_elo: Option<i32>,
}

/// User's personal record for a specific difficulty level
#[derive(Debug, Clone, FromRow, Serialize, Deserialize, SimpleObject)]
pub struct UserRecord {
    pub id: String,
    pub user_id: String,
    /// Difficulty level (1-20, Stockfish depth)
    pub difficulty: i32,
    /// Best completion time in seconds
    pub best_time_seconds: i32,
    /// Number of moves in the best game
    pub moves_count: i32,
    /// When this record was achieved
    pub achieved_at: Option<DateTime<Utc>>,
}

/// Detailed statistics for a user at a specific difficulty level
#[derive(Debug, Clone, FromRow, Serialize, Deserialize, SimpleObject)]
pub struct UserLevelStats {
    pub id: String,
    pub user_id: String,
    /// Difficulty level (1-20, Stockfish depth)
    pub difficulty: i32,
    /// Total games played at this level
    pub games_played: i32,
    /// Total games won at this level
    pub games_won: i32,
    /// Total time spent at this level (seconds)
    pub total_time_seconds: i32,
    /// Average time per game (seconds)
    pub average_time_seconds: i32,
    /// Total moves made at this level
    pub total_moves: i32,
    /// Average moves per game
    pub average_moves: i32,
}

/// Complete user profile with records and statistics
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct UserProfile {
    /// Basic user information
    pub user: User,
    /// Personal records for each difficulty level
    pub records: Vec<UserRecord>,
    /// Detailed statistics for each difficulty level
    pub level_stats: Vec<UserLevelStats>,
}