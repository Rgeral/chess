use sqlx::{SqlitePool, Row};
use crate::models::{User, Game};

pub async fn get_user_by_id(pool: &SqlitePool, user_id: &str) -> Result<Option<User>, sqlx::Error> {
    let row = sqlx::query("SELECT * FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

    if let Some(row) = row {
        Ok(Some(User {
            id: row.get("id"),
            username: row.get("username"),
            total_games: row.get("total_games"),
            games_won: row.get("games_won"),
            created_at: row.get("created_at"),
            // Nouvelles colonnes
            total_play_time_seconds: row.get("total_play_time_seconds"),
            current_streak: row.get("current_streak"),
            best_streak: row.get("best_streak"),
            estimated_elo: row.get("estimated_elo"),
        }))
    } else {
        Ok(None)
    }
}

pub async fn create_user(pool: &SqlitePool, user: &User) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO users (id, username, total_games, games_won, created_at, total_play_time_seconds, current_streak, best_streak, estimated_elo) 
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        user.id,
        user.username,
        user.total_games,
        user.games_won,
        user.created_at,
        user.total_play_time_seconds,
        user.current_streak,
        user.best_streak,
        user.estimated_elo
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_game_by_id(pool: &SqlitePool, game_id: &str) -> Result<Option<Game>, sqlx::Error> {
    let row = sqlx::query("SELECT * FROM games WHERE id = ?")
        .bind(game_id)
        .fetch_optional(pool)
        .await?;

    if let Some(row) = row {
        Ok(Some(Game {
            id: row.get("id"),
            user_id: row.get("user_id"),
            difficulty: row.get("difficulty"),
            fen: row.get("fen"),
            status: row.get("status"),
            result: row.get("result"),
            created_at: row.get("created_at"),
            start_time: row.get("start_time"),
            end_time: row.get("end_time"),
            duration_seconds: row.get("duration_seconds"),
            moves_count: row.get("moves_count"),
        }))
    } else {
        Ok(None)
    }
}

pub async fn update_game(pool: &SqlitePool, game: &Game) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE games SET fen = ?, status = ?, result = ?, end_time = ?, duration_seconds = ?, moves_count = ? WHERE id = ?",
        game.fen,
        game.status,
        game.result,
        game.end_time,
        game.duration_seconds,
        game.moves_count,
        game.id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_games_by_user(pool: &SqlitePool, user_id: &str) -> Result<Vec<Game>, sqlx::Error> {
    let rows = sqlx::query("SELECT * FROM games WHERE user_id = ? ORDER BY created_at DESC")
        .bind(user_id)
        .fetch_all(pool)
        .await?;

    let mut games = Vec::new();
    for row in rows {
        games.push(Game {
            id: row.get("id"),
            user_id: row.get("user_id"),
            difficulty: row.get("difficulty"),
            fen: row.get("fen"),
            status: row.get("status"),
            result: row.get("result"),
            created_at: row.get("created_at"),
            start_time: row.get("start_time"),
            end_time: row.get("end_time"),
            duration_seconds: row.get("duration_seconds"),
            moves_count: row.get("moves_count"),
        });
    }

    Ok(games)
}