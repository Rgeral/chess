use async_graphql::*;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, SimpleObject)]
pub struct Game {
    pub id: String,
    pub user_id: String,
    pub difficulty: i32,
    pub fen: String,
    pub status: String,
    pub result: Option<String>,
    pub created_at: DateTime<Utc>,
    // Nouvelles colonnes timer
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_seconds: Option<i32>,
    pub moves_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct GameMoveResult {
    pub game: Game,
    pub stockfish_move: String,
    pub game_over: bool,
    pub winner: Option<String>,
    // Nouvelles infos timer
    pub move_time_ms: Option<i64>,
    pub total_time_seconds: Option<i32>,
}

// Ajouter les inputs manquants
#[derive(Debug, Clone, Serialize, Deserialize, InputObject)]
pub struct NewGameInput {
    #[graphql(name = "userId")]
    pub user_id: String,
    pub difficulty: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, InputObject)]
pub struct MakeMoveInput {
    #[graphql(name = "gameId")]
    pub game_id: String,
    #[graphql(name = "playerMove")]
    pub player_move: String,
}