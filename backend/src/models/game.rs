use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use async_graphql::{SimpleObject, InputObject};

/// Represents a chess game between user and Stockfish
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, SimpleObject)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub id: String,
    #[sqlx(rename = "user_id")]
    pub user_id: String,
    pub difficulty: i32,
    pub fen: String,
    pub moves: String,
    pub status: String,
    #[sqlx(rename = "start_time")]
    pub start_time: DateTime<Utc>,
    #[sqlx(rename = "end_time")]
    pub end_time: Option<DateTime<Utc>>,
    #[sqlx(rename = "duration_seconds")]
    pub duration_seconds: Option<i32>,
}

/// Input for creating a new game
#[derive(Debug, InputObject)]
pub struct NewGameInput {
    pub user_id: String,
    pub difficulty: i32,
}

/// Input for making a move
#[derive(Debug, InputObject)]
pub struct MakeMoveInput {
    pub game_id: String,
    pub player_move: String,
}

/// Game result after a move
#[derive(Debug, SimpleObject)]
pub struct GameMoveResult {
    pub game: Game,
    pub stockfish_move: Option<String>,
    pub game_over: bool,
    pub winner: Option<String>,
}