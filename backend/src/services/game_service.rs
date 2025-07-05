use crate::models::{Game, GameMoveResult, NewGameInput, MakeMoveInput};
use crate::services::{ChessService, StockfishService, StatsService};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

/// Service responsible for managing chess games
pub struct GameService;

impl GameService {
    /// Creates a new chess game with the specified difficulty
    /// 
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `input` - Game creation parameters (user_id, difficulty)
    /// 
    /// # Returns
    /// A new Game instance initialized with starting position
    pub async fn create_game(pool: &SqlitePool, input: NewGameInput) -> Result<Game, String> {
        let game = Game {
            id: Uuid::new_v4().to_string(),
            user_id: input.user_id,
            difficulty: input.difficulty,
            fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
            status: "active".to_string(),
            result: None,
            created_at: Utc::now(),
            start_time: Some(Utc::now()),
            end_time: None,
            duration_seconds: None,
            moves_count: 0,
        };

        sqlx::query!(
            "INSERT INTO games (id, user_id, difficulty, fen, status, result, created_at, start_time, end_time, duration_seconds, moves_count) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            game.id,
            game.user_id,
            game.difficulty,
            game.fen,
            game.status,
            game.result,
            game.created_at,
            game.start_time,
            game.end_time,
            game.duration_seconds,
            game.moves_count
        )
        .execute(pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        println!("🎯 New game created: {} (Level {})", game.id, game.difficulty);
        Ok(game)
    }

    /// Retrieves a game by its ID
    /// 
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `game_id` - Unique identifier of the game
    /// 
    /// # Returns
    /// Option<Game> - Some(game) if found, None if not found
    pub async fn get_game(pool: &SqlitePool, game_id: &str) -> Result<Option<Game>, String> {
        let row = sqlx::query!(
            "SELECT id, user_id, difficulty, fen, status, result, created_at, start_time, end_time, duration_seconds, moves_count FROM games WHERE id = ?",
            game_id
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        if let Some(row) = row {
let game = Game {
    id: row.id,
    user_id: row.user_id,
    difficulty: row.difficulty as i32,
    fen: row.fen,
    status: row.status,
    result: row.result,
    created_at: DateTime::<Utc>::from_naive_utc_and_offset(row.created_at, Utc),
    start_time: row.start_time.map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc)),
    end_time: row.end_time.map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc)),
    duration_seconds: row.duration_seconds.map(|d| d as i32),
    moves_count: row.moves_count as i32,
};
            Ok(Some(game))
        } else {
            Ok(None)
        }
    }

    /// Retrieves all games for a specific user
    /// 
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `user_id` - Unique identifier of the user
    /// 
    /// # Returns
    /// Vector of games ordered by creation date (newest first)
    pub async fn get_user_games(pool: &SqlitePool, user_id: &str) -> Result<Vec<Game>, String> {
        let rows = sqlx::query!(
            "SELECT id, user_id, difficulty, fen, status, result, created_at, start_time, end_time, duration_seconds, moves_count FROM games WHERE user_id = ? ORDER BY created_at DESC",
            user_id
        )
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        let games = rows
            .into_iter()
            .map(|row| Game {
                id: row.id,
                user_id: row.user_id,
                difficulty: row.difficulty as i32,
                fen: row.fen,
                status: row.status,
                result: row.result,
                created_at: DateTime::<Utc>::from_naive_utc_and_offset(row.created_at, Utc),
                start_time: row.start_time.map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc)),
                end_time: row.end_time.map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc)),
                duration_seconds: row.duration_seconds.map(|d| d as i32),
                moves_count: row.moves_count as i32,
            })
            .collect();

        Ok(games)
    }

    /// Processes a player's move and generates Stockfish response
    /// 
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `input` - Move input containing game_id and player_move in algebraic notation
    /// 
    /// # Returns
    /// GameMoveResult containing updated game state and Stockfish's response
    /// 
    /// # Process
    /// 1. Validates and applies player's move
    /// 2. Checks if game ends after player's move
    /// 3. If game continues, gets Stockfish's response
    /// 4. Checks if game ends after Stockfish's move
    /// 5. Updates statistics if game finishes
    /// 6. Saves updated game state to database
    pub async fn make_move(pool: &SqlitePool, input: MakeMoveInput) -> Result<GameMoveResult, String> {
        println!("🎮 Processing move: {} in game {}", input.player_move, input.game_id);

        // Fetch current game state
        let row = sqlx::query!(
            "SELECT id, user_id, difficulty, fen, status, result, created_at, start_time, end_time, duration_seconds, moves_count FROM games WHERE id = ?",
            input.game_id
        )
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Game not found: {}", e))?;

        let mut game = Game {
            id: row.id,
            user_id: row.user_id,
            difficulty: row.difficulty as i32,
            fen: row.fen,
            status: row.status,
            result: row.result,
            created_at: DateTime::<Utc>::from_naive_utc_and_offset(row.created_at, Utc),
            start_time: row.start_time.map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc)),
            end_time: row.end_time.map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc)),
            duration_seconds: row.duration_seconds.map(|d| d as i32),
            moves_count: row.moves_count as i32,
        };

        if game.status != "active" {
            return Err("Game is not active".to_string());
        }

        // Apply player's move
        let new_fen = ChessService::make_move(&game.fen, &input.player_move)
            .map_err(|e| format!("Illegal move: {}", e))?;

        game.moves_count += 1;
        game.fen = new_fen.clone();

        // Check if game ends after player's move
        let (game_over, winner) = ChessService::check_game_over(&new_fen);
        
        let stockfish_move: String;
        let mut last_move_piece_type = "unknown".to_string();
        
        if game_over {
            // Game ends, update final state
            game.status = "finished".to_string();
            game.result = winner.clone();
            game.end_time = Some(Utc::now());
            
            if let Some(start_time) = game.start_time {
                let duration = (Utc::now() - start_time).num_seconds() as i32;
                game.duration_seconds = Some(duration);
                
                let won = winner.as_deref() == Some("white");
                StatsService::update_game_stats(
                    pool,
                    &game.user_id,
                    game.difficulty,
                    duration,
                    game.moves_count,
                    won,
                ).await.map_err(|e| format!("Stats update error: {}", e))?;
            }
            
            stockfish_move = "none".to_string();
            println!("🏁 Game finished! Winner: {:?}", winner);
        } else {
            // Game continues, get Stockfish response
            stockfish_move = StockfishService::get_best_move(&new_fen, game.difficulty)
                .await
                .map_err(|e| format!("Stockfish error: {}", e))?;

            println!("🤖 Stockfish plays: {}", stockfish_move);
            println!("🎯 Move breakdown: from='{}', to='{}'", &stockfish_move[0..2], &stockfish_move[2..4]);

            // Get the piece type that's moving BEFORE applying the move
            let piece_type = if stockfish_move.len() >= 4 {
                let from_square = &stockfish_move[0..2];
                println!("🔍 Looking for piece at square: {}", from_square);
                // Utiliser la FEN AVANT le coup de Stockfish (c'est game.fen ici)
                let piece = ChessService::get_piece_at_square(&game.fen, from_square)
                    .unwrap_or_else(|| "piece".to_string());
                println!("🎲 Found piece: {}", piece);
                piece
            } else {
                "unknown".to_string()
            };

            // Apply Stockfish's move
            game.fen = ChessService::make_move(&new_fen, &stockfish_move)
                .map_err(|e| format!("Stockfish move error: {}", e))?;

            game.moves_count += 1;

            // Check if game ends after Stockfish's move
            let (sf_game_over, sf_winner) = ChessService::check_game_over(&game.fen);
            if sf_game_over {
                game.status = "finished".to_string();
                game.result = sf_winner.clone();
                game.end_time = Some(Utc::now());
                
                if let Some(start_time) = game.start_time {
                    let duration = (Utc::now() - start_time).num_seconds() as i32;
                    game.duration_seconds = Some(duration);
                    
                    let won = sf_winner.as_deref() == Some("white");
                    StatsService::update_game_stats(
                        pool,
                        &game.user_id,
                        game.difficulty,
                        duration,
                        game.moves_count,
                        won,
                    ).await.map_err(|e| format!("Stats update error: {}", e))?;
                }
                println!("🏁 Game finished after Stockfish move! Winner: {:?}", sf_winner);
            }

            // Store the piece type for animation
            last_move_piece_type = piece_type;
        }

        // Save updated game state
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
        .await
        .map_err(|e| format!("Database update error: {}", e))?;

    let total_time_seconds = if let Some(start_time) = game.start_time {
        Some((Utc::now() - start_time).num_seconds() as i32)
    } else {
        None
    };

    // Clone le result AVANT de déplacer game
    let winner = game.result.clone();
    let game_over = game.status == "finished";

    // Créer lastMove pour l'animation avec le coup de Stockfish (seul coup à animer)
    let last_move = if stockfish_move != "none" {
        let from_square = stockfish_move[0..2].to_string();
        let to_square = stockfish_move[2..4].to_string();
        println!("🎬 Creating LastMove: {} -> {} (piece: {}, color: black)", from_square, to_square, last_move_piece_type);
        Some(crate::models::game::LastMove {
            from: from_square,
            to: to_square,
            piece: last_move_piece_type,
            color: "black".to_string(),
        })
    } else {
        None
    };

    println!("✅ Move processed successfully");

    Ok(GameMoveResult {
        game,
        stockfish_move,
        game_over,
        winner,
        move_time_ms: None,
        total_time_seconds,
        last_move,
    })
    }
}