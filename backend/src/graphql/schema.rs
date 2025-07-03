use async_graphql::*;
use crate::models::{User, Game, NewGameInput, MakeMoveInput, GameMoveResult, UserProfile}; // Ajouter UserProfile
use sqlx::SqlitePool;
use crate::services::{UserService, GameService, StatsService};
use crate::database::*;


/// GraphQL Query root - handles all read operations
pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Retrieves a user by their ID
    /// Returns None if user doesn't exist
    async fn user(&self, ctx: &Context<'_>, id: String) -> Result<Option<User>, Error> {
        let db = ctx.data::<SqlitePool>()?;
        let user = get_user_by_id(db, &id).await?;
        Ok(user)
    }

    /// Retrieves a specific game by its ID
    /// Returns None if game doesn't exist
    async fn game(&self, ctx: &Context<'_>, game_id: String) -> Result<Option<Game>, Error> {
        let db = ctx.data::<SqlitePool>()?;
        let game = GameService::get_game(db, &game_id).await?;
        Ok(game)
    }

        async fn get_user_profile(&self, ctx: &Context<'_>, user_id: String) -> Result<UserProfile, Error> {
        let db = ctx.data::<SqlitePool>()?;
        let profile = StatsService::get_user_profile(db, &user_id).await
            .map_err(|e| Error::new(format!("Database error: {}", e)))?;
        Ok(profile)
    }

    /// Get leaderboard (top players by ELO)
    async fn get_leaderboard(&self, ctx: &Context<'_>, limit: Option<i32>) -> Result<Vec<User>, Error> {
        let db = ctx.data::<SqlitePool>()?;
        let limit = limit.unwrap_or(10);
        
        let rows = sqlx::query!(
            "SELECT id, username, total_games, games_won, created_at, 
                    total_play_time_seconds, current_streak, best_streak, estimated_elo 
             FROM users 
             WHERE estimated_elo IS NOT NULL 
             ORDER BY estimated_elo DESC 
             LIMIT ?",
            limit
        )
        .fetch_all(db)
        .await
        .map_err(|e| Error::new(format!("Database error: {}", e)))?;

        let users = rows.into_iter().map(|row| User {
            id: row.id,
            username: row.username,
            total_games: row.total_games as i32,
            games_won: row.games_won as i32,
            created_at: chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(row.created_at, chrono::Utc),
            total_play_time_seconds: row.total_play_time_seconds.map(|v| v as i32),
            current_streak: row.current_streak.map(|v| v as i32),
            best_streak: row.best_streak.map(|v| v as i32),
            estimated_elo: row.estimated_elo.map(|v| v as i32),
        }).collect();

        Ok(users)
    }

    /// Get user's current ELO rating
    async fn get_user_elo(&self, ctx: &Context<'_>, user_id: String) -> Result<i32, Error> {
        let db = ctx.data::<SqlitePool>()?;
        let elo = StatsService::estimate_player_elo(db, &user_id).await
            .map_err(|e| Error::new(format!("Database error: {}", e)))?;
        Ok(elo)
    }

    /// Retrieves all games for a specific user
    /// Returns empty vector if user has no games
    async fn user_games(&self, ctx: &Context<'_>, user_id: String) -> Result<Vec<Game>, Error> {
        let db = ctx.data::<SqlitePool>()?;
        let games = GameService::get_user_games(db, &user_id).await?;
        Ok(games)
    }

    /// Simple health check endpoint
    async fn hello(&self) -> &str {
        "Hello from Chess GraphQL API!"
    }
}

/// GraphQL Mutation root - handles all write operations
pub struct MutationRoot;

#[Object]
impl MutationRoot {
    /// Creates a new user with the given username
    /// Returns existing user if username already exists
    async fn create_user(&self, ctx: &Context<'_>, username: String) -> Result<User, Error> {
        let db = ctx.data::<SqlitePool>()?;
        let user = UserService::create_user(username);
        create_user(db, &user).await?;
        Ok(user)
    }

    /// Creates a new chess game with specified difficulty
    /// Initializes the game with standard starting position
    async fn create_game(&self, ctx: &Context<'_>, input: NewGameInput) -> Result<Game, Error> {
        let db = ctx.data::<SqlitePool>()?;
        let game = GameService::create_game(db, input).await?;
        Ok(game)
    }

    /// Makes a move in an existing game
    /// Validates the move, applies it, and gets Stockfish response
    /// Updates game statistics if game ends
    async fn make_move(&self, ctx: &Context<'_>, input: MakeMoveInput) -> Result<GameMoveResult, Error> {
        let db = ctx.data::<SqlitePool>()?;
        let result = GameService::make_move(db, input).await?;
        Ok(result)
    }
}