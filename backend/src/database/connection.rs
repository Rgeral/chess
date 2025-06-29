use sqlx::{SqlitePool, Row};
use crate::models::{User, Game};
use uuid::Uuid;
use chrono::Utc;

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = SqlitePool::connect(database_url).await?;
        Ok(Database { pool })
    }
    
    /// Find user by username
    pub async fn find_user_by_username(&self, username: &str) -> Result<Option<User>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, username, created_at, total_games, games_won, games_lost, games_draw, max_difficulty_beaten, last_played 
             FROM users WHERE username = ?"
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some(row) = row {
            let user = User {
                id: row.get("id"),
                username: row.get("username"),
                created_at: row.get("created_at"),
                total_games: row.get("total_games"),
                games_won: row.get("games_won"),
                games_lost: row.get("games_lost"),
                games_draw: row.get("games_draw"),
                max_difficulty_beaten: row.get("max_difficulty_beaten"),
                last_played: row.get("last_played"),
            };
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }
    
    /// Create new user
    pub async fn create_user(&self, username: &str) -> Result<User, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        let created_at = Utc::now();
        
        sqlx::query(
            "INSERT INTO users (id, username, created_at, total_games, games_won, games_lost, games_draw, max_difficulty_beaten, last_played) 
             VALUES (?, ?, ?, 0, 0, 0, 0, 0, NULL)"
        )
        .bind(&id)
        .bind(username)
        .bind(&created_at)
        .execute(&self.pool)
        .await?;
        
        self.find_user_by_username(username)
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    /// Create a new game
    pub async fn create_game(&self, game: &Game) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO games (id, user_id, difficulty, fen, moves, status, start_time, end_time, duration_seconds)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&game.id)
        .bind(&game.user_id)
        .bind(&game.difficulty)
        .bind(&game.fen)
        .bind(&game.moves)
        .bind(&game.status)
        .bind(&game.start_time)
        .bind(&game.end_time)
        .bind(&game.duration_seconds)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    /// Find game by ID
    pub async fn find_game_by_id(&self, game_id: &str) -> Result<Option<Game>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, user_id, difficulty, fen, moves, status, start_time, end_time, duration_seconds
             FROM games WHERE id = ?"
        )
        .bind(game_id)
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some(row) = row {
            let game = Game {
                id: row.get("id"),
                user_id: row.get("user_id"),
                difficulty: row.get("difficulty"),
                fen: row.get("fen"),
                moves: row.get("moves"),
                status: row.get("status"),
                start_time: row.get("start_time"),
                end_time: row.get("end_time"),
                duration_seconds: row.get("duration_seconds"),
            };
            Ok(Some(game))
        } else {
            Ok(None)
        }
    }

    /// Update game after move
    pub async fn update_game(&self, game: &Game) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE games SET fen = ?, moves = ?, status = ?, end_time = ?, duration_seconds = ? WHERE id = ?"
        )
        .bind(&game.fen)
        .bind(&game.moves)
        .bind(&game.status)
        .bind(&game.end_time)
        .bind(&game.duration_seconds)
        .bind(&game.id)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    /// Get user's games
    pub async fn get_user_games(&self, user_id: &str) -> Result<Vec<Game>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, user_id, difficulty, fen, moves, status, start_time, end_time, duration_seconds
             FROM games WHERE user_id = ? ORDER BY start_time DESC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        
        let games = rows.into_iter().map(|row| Game {
            id: row.get("id"),
            user_id: row.get("user_id"),
            difficulty: row.get("difficulty"),
            fen: row.get("fen"),
            moves: row.get("moves"),
            status: row.get("status"),
            start_time: row.get("start_time"),
            end_time: row.get("end_time"),
            duration_seconds: row.get("duration_seconds"),
        }).collect();
        
        Ok(games)
    }
}