use sqlx::{PgPool, Row};
use crate::models::{User, Game};

pub async fn get_user_by_id(pool: &PgPool, user_id: &str) -> Result<Option<User>, sqlx::Error> {
    let row = sqlx::query("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

    if let Some(row) = row {
        Ok(Some(User {
            id: row.get("id"),
            username: row.get("username"),
            
            total_games: row.get::<i32, _>("total_games") as i32,
            games_won: row.get::<i32, _>("games_won") as i32,
            created_at: row.get("created_at"),
            // Option integers: map i64 -> i32
            total_play_time_seconds: row.get::<Option<i32>, _>("total_play_time_seconds").map(|v| v as i32),
            current_streak: row.get::<Option<i32>, _>("current_streak").map(|v| v as i32),
            best_streak: row.get::<Option<i32>, _>("best_streak").map(|v| v as i32),
            estimated_elo: row.get::<Option<i32>, _>("estimated_elo").map(|v| v as i32),
        }))
    } else {
        Ok(None)
    }
}

pub async fn create_user(pool: &PgPool, user: &User) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO users (id, username, total_games, games_won, created_at, total_play_time_seconds, current_streak, best_streak, estimated_elo) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
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

pub async fn get_game_by_id(pool: &PgPool, game_id: &str) -> Result<Option<Game>, sqlx::Error> {
    let row = sqlx::query("SELECT * FROM games WHERE id = $1")
        .bind(game_id)
        .fetch_optional(pool)
        .await?;

    if let Some(row) = row {
        Ok(Some(Game {
            id: row.get("id"),
            user_id: row.get("user_id"),
            difficulty: row.get::<i32, _>("difficulty") as i32,
            fen: row.get("fen"),
            status: row.get("status"),
            result: row.get("result"),
            created_at: row.get("created_at"),
            start_time: row.get("start_time"),
            end_time: row.get("end_time"),
            duration_seconds: row.get::<Option<i32>, _>("duration_seconds").map(|v| v as i32),
            moves_count: row.get::<i32, _>("moves_count") as i32,
        }))
    } else {
        Ok(None)
    }
}

pub async fn update_game(pool: &PgPool, game: &Game) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE games SET fen = $1, status = $2, result = $3, end_time = $4, duration_seconds = $5, moves_count = $6 WHERE id = $7",
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

pub async fn get_games_by_user(pool: &PgPool, user_id: &str) -> Result<Vec<Game>, sqlx::Error> {
    let rows = sqlx::query("SELECT * FROM games WHERE user_id = $1 ORDER BY created_at DESC")
        .bind(user_id)
        .fetch_all(pool)
        .await?;

    let mut games = Vec::new();
    for row in rows {
        games.push(Game {
            id: row.get("id"),
            user_id: row.get("user_id"),
            difficulty: row.get::<i32, _>("difficulty") as i32,
            fen: row.get("fen"),
            status: row.get("status"),
            result: row.get("result"),
            created_at: row.get("created_at"),
            start_time: row.get("start_time"),
            end_time: row.get("end_time"),
            duration_seconds: row.get::<Option<i32>, _>("duration_seconds").map(|v| v as i32),
            moves_count: row.get::<i32, _>("moves_count") as i32,
        });
    }

    Ok(games)
}