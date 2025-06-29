use crate::models::{Game, NewGameInput, MakeMoveInput, GameMoveResult};
use crate::database::Database;
use crate::services::{StockfishService, ChessService, GameStatus};
use uuid::Uuid;
use chrono::Utc;

/// Service for managing chess games
pub struct GameService;

impl GameService {
    /// Create a new game against Stockfish
    pub async fn create_game(db: &Database, input: NewGameInput) -> Result<Game, String> {
        if input.difficulty < 1 || input.difficulty > 20 {
            return Err("Difficulty must be between 1 and 20".to_string());
        }
        
        let game = Game {
            id: Uuid::new_v4().to_string(),
            user_id: input.user_id,
            difficulty: input.difficulty,
            fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
            moves: "[]".to_string(), // JSON array of moves
            status: "playing".to_string(),
            start_time: Utc::now(),
            end_time: None,
            duration_seconds: None,
        };
        
        db.create_game(&game).await.map_err(|e| e.to_string())?;
        println!("✅ Created game {} vs Stockfish difficulty {}", game.id, game.difficulty);
        Ok(game)
    }
    
    /// Get game by ID
    pub async fn get_game(db: &Database, game_id: String) -> Result<Option<Game>, String> {
        db.find_game_by_id(&game_id).await.map_err(|e| e.to_string())
    }
    
    /// Get user's games
    pub async fn get_user_games(db: &Database, user_id: String) -> Result<Vec<Game>, String> {
        db.get_user_games(&user_id).await.map_err(|e| e.to_string())
    }

    /// Make a move in the game with real chess validation
    pub async fn make_move(db: &Database, input: MakeMoveInput) -> Result<GameMoveResult, String> {
        // 1. Get current game
        let mut game = db.find_game_by_id(&input.game_id).await
            .map_err(|e| e.to_string())?
            .ok_or("Game not found")?;
        
        if game.status != "playing" {
            return Err("Game is already finished".to_string());
        }
        
        // 2. Validate player move
        if !ChessService::validate_move(&game.fen, &input.player_move)? {
            return Err(format!("Illegal move: {}", input.player_move));
        }
        
        // 3. Apply player move to get new position
        let new_fen = ChessService::apply_move(&game.fen, &input.player_move)?;
        game.fen = new_fen;
        
        // 4. Update moves history
        let mut moves: Vec<String> = serde_json::from_str(&game.moves)
            .unwrap_or_else(|_| vec![]);
        moves.push(input.player_move.clone());
        
        // 5. Check if game is over after player move
        let status_after_player = ChessService::check_game_status(&game.fen)?;
        let mut stockfish_move = None;
        let mut game_over = false;
        let mut winner = None;
        
        match status_after_player {
            GameStatus::Checkmate(w) => {
                game_over = true;
                winner = Some(w);
                game.status = "finished".to_string();
            },
            GameStatus::Stalemate | GameStatus::Draw => {
                game_over = true;
                winner = Some("draw".to_string());
                game.status = "draw".to_string();
            },
            GameStatus::Playing => {
                // 6. Get Stockfish response
                match StockfishService::get_best_move(&game.fen, game.difficulty).await {
                    Ok(sf_move) => {
                        // Apply Stockfish move
                        game.fen = ChessService::apply_move(&game.fen, &sf_move)?;
                        moves.push(sf_move.clone());
                        stockfish_move = Some(sf_move);
                        
                        // Check game status after Stockfish move
                        match ChessService::check_game_status(&game.fen)? {
                            GameStatus::Checkmate(w) => {
                                game_over = true;
                                winner = Some(w);
                                game.status = "finished".to_string();
                            },
                            GameStatus::Stalemate | GameStatus::Draw => {
                                game_over = true;
                                winner = Some("draw".to_string());
                                game.status = "draw".to_string();
                            },
                            GameStatus::Playing => {
                                // Game continues
                            }
                        }
                    },
                    Err(e) => {
                        println!("⚠️ Stockfish error: {}", e);
                        // Continue without Stockfish move
                    }
                }
            }
        }
        
        // 7. Update game record
        game.moves = serde_json::to_string(&moves).unwrap();
        
        if game_over {
            game.end_time = Some(Utc::now());
            game.duration_seconds = Some(
                (game.end_time.unwrap() - game.start_time).num_seconds() as i32
            );
        }
        
        // 8. Save to database
        db.update_game(&game).await.map_err(|e| e.to_string())?;
        
        println!("♟️ Move: {} -> {} (Status: {:?})", 
                input.player_move, 
                stockfish_move.as_deref().unwrap_or("none"),
                game.status);
        
        Ok(GameMoveResult {
            game,
            stockfish_move,
            game_over,
            winner,
        })
    }
}