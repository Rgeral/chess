use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use async_graphql::SimpleObject;

/// Represents a chess player with their game statistics
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, SimpleObject)]
pub struct User {
    pub id: String,
    pub username: String,
    pub created_at: DateTime<Utc>,
    pub total_games: i32,
    pub games_won: i32,
    pub games_lost: i32,
    pub games_draw: i32,
    pub max_difficulty_beaten: i32,
    pub last_played: Option<DateTime<Utc>>,
}