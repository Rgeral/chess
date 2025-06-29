use chess::{Board, ChessMove, Color, MoveGen};
use std::str::FromStr;

/// Service for chess game logic and validation
pub struct ChessService;

impl ChessService {
    /// Validate if a move is legal in the given position
    pub fn validate_move(fen: &str, move_str: &str) -> Result<bool, String> {
        // Parse FEN to get board position
        let board = Board::from_str(fen)
            .map_err(|e| format!("Invalid FEN: {}", e))?;
        
        // Parse move string (e.g., "e2e4")
        let chess_move = ChessMove::from_str(move_str)
            .map_err(|e| format!("Invalid move format: {}", e))?;
        
        // Check if move is legal using the board's validate method
        Ok(board.legal(chess_move))
    }
    
    /// Apply a move to a position and return new FEN
    pub fn apply_move(fen: &str, move_str: &str) -> Result<String, String> {
        let board = Board::from_str(fen)
            .map_err(|e| format!("Invalid FEN: {}", e))?;
        
        let chess_move = ChessMove::from_str(move_str)
            .map_err(|e| format!("Invalid move format: {}", e))?;
        
        // Validate move is legal
        if !board.legal(chess_move) {
            return Err(format!("Illegal move: {}", move_str));
        }
        
        // Apply move and get new board
        let new_board = board.make_move_new(chess_move);
        
        Ok(new_board.to_string())
    }
    
    /// Check if position is checkmate, stalemate, or draw
    pub fn check_game_status(fen: &str) -> Result<GameStatus, String> {
        let board = Board::from_str(fen)
            .map_err(|e| format!("Invalid FEN: {}", e))?;
        
        match board.status() {
            chess::BoardStatus::Checkmate => {
                let winner = match board.side_to_move() {
                    Color::White => "black", // If white to move and checkmate, black wins
                    Color::Black => "white",
                };
                Ok(GameStatus::Checkmate(winner.to_string()))
            },
            chess::BoardStatus::Stalemate => Ok(GameStatus::Stalemate),
            chess::BoardStatus::Ongoing => Ok(GameStatus::Playing),
        }
    }
    
    /// Get all legal moves for current position
    pub fn get_legal_moves(fen: &str) -> Result<Vec<String>, String> {
        let board = Board::from_str(fen)
            .map_err(|e| format!("Invalid FEN: {}", e))?;
        
        // Utiliser MoveGen pour générer les coups légaux
        let movegen = MoveGen::new_legal(&board);
        let moves: Vec<String> = movegen
            .map(|mv| mv.to_string())
            .collect();
        
        Ok(moves)
    }
}

#[derive(Debug)]
pub enum GameStatus {
    Playing,
    Checkmate(String), // Winner
    Stalemate,
    Draw,
}