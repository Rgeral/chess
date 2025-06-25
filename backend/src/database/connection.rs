use sqlx::{SqlitePool, Row};
use crate::models::User;
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
    
    /// Find user by username - Version sans macros pour éviter les problèmes de types
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
    
    /// Create new user - Version simple
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
        
        // Return the created user
        self.find_user_by_username(username)
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }
}