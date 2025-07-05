use sqlx::SqlitePool;
use uuid::Uuid;
use crate::models::{User, UserRecord, UserLevelStats, UserProfile};

/// Service responsible for managing user statistics and records
pub struct StatsService;

impl StatsService {
    /// Updates user statistics after a game completion
    /// 
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `user_id` - User identifier
    /// * `difficulty` - Game difficulty level (1-20)
    /// * `duration_seconds` - Total game duration in seconds
    /// * `moves_count` - Total moves made in the game
    /// * `won` - Whether the user won the game
    /// 
    /// # Updates
    /// - User's total play time and streaks
    /// - Level-specific statistics (games played, won, time, moves)
    /// - Personal records if applicable
    pub async fn update_game_stats(
        pool: &SqlitePool,
        user_id: &str,
        difficulty: i32,
        duration_seconds: i32,
        moves_count: i32,
        won: bool,
    ) -> Result<(), sqlx::Error> {
        println!("📊 Updating stats for user {} - Level {}, Time: {}s, Moves: {}, Won: {}",
                 user_id, difficulty, duration_seconds, moves_count, won);

        let mut tx = pool.begin().await?;

        // Update user's total play time
        sqlx::query!(
            "UPDATE users SET total_play_time_seconds = COALESCE(total_play_time_seconds, 0) + ? WHERE id = ?",
            duration_seconds,
            user_id
        )
        .execute(&mut *tx)
        .await?;

        // Update user's win streaks
        if won {
            sqlx::query!(
                "UPDATE users SET 
                    current_streak = COALESCE(current_streak, 0) + 1,
                    best_streak = MAX(COALESCE(best_streak, 0), COALESCE(current_streak, 0) + 1)
                WHERE id = ?",
                user_id
            )
            .execute(&mut *tx)
            .await?;
        } else {
            sqlx::query!(
                "UPDATE users SET current_streak = 0 WHERE id = ?",
                user_id
            )
            .execute(&mut *tx)
            .await?;
        }

        // Create variables to avoid temporary values
        let stats_id = Uuid::new_v4().to_string();
        let games_won_val = if won { 1 } else { 0 };
        
        // Update level-specific statistics
        sqlx::query!(
            "INSERT INTO user_level_stats (id, user_id, difficulty, games_played, games_won, total_time_seconds, total_moves)
             VALUES (?, ?, ?, 1, ?, ?, ?)
             ON CONFLICT(user_id, difficulty) DO UPDATE SET
                games_played = games_played + 1,
                games_won = games_won + ?,
                total_time_seconds = total_time_seconds + ?,
                total_moves = total_moves + ?,
                average_time_seconds = (total_time_seconds + ?) / (games_played + 1),
                average_moves = (total_moves + ?) / (games_played + 1)",
            stats_id,
            user_id,
            difficulty,
            games_won_val,
            duration_seconds,
            moves_count,
            games_won_val,
            duration_seconds,
            moves_count,
            duration_seconds,
            moves_count
        )
        .execute(&mut *tx)
        .await?;

        // Update personal record if this was a win
        if won {
            Self::update_personal_record(&mut tx, user_id, difficulty, duration_seconds, moves_count).await?;
        }

        tx.commit().await?;
        println!("✅ Stats updated successfully");
        Ok(())
    }

    /// Updates a user's personal record for a difficulty level
    /// 
    /// # Arguments
    /// * `tx` - Database transaction
    /// * `user_id` - User identifier
    /// * `difficulty` - Difficulty level
    /// * `duration_seconds` - Game completion time
    /// * `moves_count` - Number of moves made
    /// 
    /// Only updates if this is a new record (faster completion time)
    async fn update_personal_record(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        user_id: &str,
        difficulty: i32,
        duration_seconds: i32,
        moves_count: i32,
    ) -> Result<(), sqlx::Error> {
        // Check current record for this difficulty
        let current_record = sqlx::query!(
            "SELECT best_time_seconds FROM user_records WHERE user_id = ? AND difficulty = ?",
            user_id,
            difficulty
        )
        .fetch_optional(&mut **tx)
        .await?;

        let is_new_record = match current_record {
            Some(record) => duration_seconds < (record.best_time_seconds as i32),
            None => true, // First record for this difficulty
        };

        if is_new_record {
            let record_id = Uuid::new_v4().to_string();
            sqlx::query!(
                "INSERT INTO user_records (id, user_id, difficulty, best_time_seconds, moves_count)
                 VALUES (?, ?, ?, ?, ?)
                 ON CONFLICT(user_id, difficulty) DO UPDATE SET
                    best_time_seconds = ?,
                    moves_count = ?,
                    achieved_at = CURRENT_TIMESTAMP",
                record_id,
                user_id,
                difficulty,
                duration_seconds,
                moves_count,
                duration_seconds,
                moves_count
            )
            .execute(&mut **tx)
            .await?;

            println!("🏆 NEW RECORD! Level {} completed in {}s with {} moves", 
                     difficulty, duration_seconds, moves_count);
        }

        Ok(())
    }

    /// Retrieves a complete user profile with statistics and records
    /// 
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `user_id` - User identifier
    /// 
    /// # Returns
    /// UserProfile containing basic user info, personal records, and level statistics
    pub async fn get_user_profile(pool: &SqlitePool, user_id: &str) -> Result<UserProfile, sqlx::Error> {
        // Fetch user data
        let user_row = sqlx::query!(
            "SELECT id, username, total_games, games_won, created_at, 
                    total_play_time_seconds, current_streak, best_streak, estimated_elo 
             FROM users WHERE id = ?",
            user_id
        )
        .fetch_one(pool)
        .await?;

let user = User {
    id: user_row.id,
    username: user_row.username,
    total_games: user_row.total_games as i32,
    games_won: user_row.games_won as i32,
    created_at: chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(user_row.created_at, chrono::Utc),
    total_play_time_seconds: user_row.total_play_time_seconds.map(|v| v as i32),
    current_streak: user_row.current_streak.map(|v| v as i32),
    best_streak: user_row.best_streak.map(|v| v as i32),
    estimated_elo: user_row.estimated_elo.map(|v| v as i32),
};

        // Fetch personal records
        let record_rows = sqlx::query!(
            "SELECT id, user_id, difficulty, best_time_seconds, moves_count, achieved_at 
            FROM user_records WHERE user_id = ? ORDER BY difficulty ASC",
            user_id
        )
        .fetch_all(pool)
        .await?;

        let records: Vec<UserRecord> = record_rows.into_iter().map(|row| UserRecord {
            id: row.id,
            user_id: row.user_id,
            difficulty: row.difficulty as i32,
            best_time_seconds: row.best_time_seconds as i32,
            moves_count: row.moves_count as i32,
            achieved_at: Some(chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(row.achieved_at, chrono::Utc)),
        }).collect();
        // Fetch level statistics
        let stats_rows = sqlx::query!(
            "SELECT id, user_id, difficulty, games_played, games_won, 
                    total_time_seconds, average_time_seconds, total_moves, average_moves 
             FROM user_level_stats WHERE user_id = ? ORDER BY difficulty ASC",
            user_id
        )
        .fetch_all(pool)
        .await?;

        let level_stats: Vec<UserLevelStats> = stats_rows.into_iter().map(|row| UserLevelStats {
            id: row.id,
            user_id: row.user_id,
            difficulty: row.difficulty as i32,
            games_played: row.games_played as i32,
            games_won: row.games_won as i32,
            total_time_seconds: row.total_time_seconds as i32,
            average_time_seconds: row.average_time_seconds as i32,
            total_moves: row.total_moves as i32,
            average_moves: row.average_moves as i32,
        }).collect();

        Ok(UserProfile {
            user,
            records,
            level_stats,
        })
    }

    /// Estimates a player's ELO rating based on their performance
    /// 
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `user_id` - User identifier
    /// 
    /// # Returns
    /// Estimated ELO rating (800-3000+ range)
    /// 
    /// # Algorithm
    /// - Base ELO is 800
    /// - For each difficulty level with >50% win rate, player is considered to be at that level
    /// - Each difficulty level corresponds to ~100 ELO points
    pub async fn estimate_player_elo(pool: &SqlitePool, user_id: &str) -> Result<i32, sqlx::Error> {
        let stats = sqlx::query!(
            "SELECT difficulty, games_played, games_won 
             FROM user_level_stats 
             WHERE user_id = ? AND games_played > 0",
            user_id
        )
        .fetch_all(pool)
        .await?;

        if stats.is_empty() {
            return Ok(800); // Base ELO for new players
        }

        let mut estimated_elo = 800i64;
        
        for stat in stats {
let games_won = stat.games_won as f64;
let games_played = stat.games_played as f64;
let win_rate = games_won / games_played;
let level_elo = stat.difficulty as i64 * 100;
            
            // If win rate >= 50%, player can handle this difficulty level
            if win_rate >= 0.5 {
                estimated_elo = estimated_elo.max(level_elo);
            }
        }

        let final_elo = estimated_elo as i32;

        // Update user's estimated ELO in database
        sqlx::query!(
            "UPDATE users SET estimated_elo = ? WHERE id = ?",
            final_elo,
            user_id
        )
        .execute(pool)
        .await?;

        println!("📈 Updated ELO for user {}: {}", user_id, final_elo);
        Ok(final_elo)
    }
}