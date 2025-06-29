use chess::{Board, ChessMove, Color, MoveGen};
use std::str::FromStr;

/// Service responsible for chess game logic and move validation
pub struct ChessService;

impl ChessService {
    /// Parses and validates a chess move in algebraic notation
    /// 
    /// # Arguments
    /// * `fen` - Current board position in FEN notation
    /// * `move_str` - Move in algebraic notation (e.g., "e2e4", "Nf3")
    /// 
    /// # Returns
    /// Result<ChessMove, String> - Valid move or error message
    pub fn parse_move(fen: &str, move_str: &str) -> Result<ChessMove, String> {
        let board = Board::from_str(fen).map_err(|e| format!("Invalid FEN: {}", e))?;
        
        let chess_move = ChessMove::from_str(move_str)
            .map_err(|e| format!("Invalid move format: {}", e))?;
        
        if board.legal(chess_move) {
            Ok(chess_move)
        } else {
            Err("Illegal move".to_string())
        }
    }

    /// Applies a chess move to a position and returns the new position
    /// 
    /// # Arguments
    /// * `fen` - Current board position in FEN notation
    /// * `move_str` - Move in algebraic notation
    /// 
    /// # Returns
    /// Result<String, String> - New FEN position or error message
    pub fn make_move(fen: &str, move_str: &str) -> Result<String, String> {
        let mut board = Board::from_str(fen).map_err(|e| format!("Invalid FEN: {}", e))?;
        
        let chess_move = ChessMove::from_str(move_str)
            .map_err(|e| format!("Invalid move format: {}", e))?;
        
        if !board.legal(chess_move) {
            return Err("Illegal move".to_string());
        }
        
        board = board.make_move_new(chess_move);
        Ok(board.to_string())
    }

    /// Checks if the game is over and determines the winner
    /// 
    /// # Arguments
    /// * `fen` - Current board position in FEN notation
    /// 
    /// # Returns
    /// (bool, Option<String>) - (is_game_over, winner)
    /// - winner: "white", "black", or "draw"
    pub fn check_game_over(fen: &str) -> (bool, Option<String>) {
        let board = match Board::from_str(fen) {
            Ok(board) => board,
            Err(_) => return (false, None),
        };
        
        match board.status() {
            chess::BoardStatus::Checkmate => {
                let winner = match board.side_to_move() {
                    Color::White => "black", // White to move but checkmate = Black wins
                    Color::Black => "white", // Black to move but checkmate = White wins
                };
                (true, Some(winner.to_string()))
            },
            chess::BoardStatus::Stalemate => (true, Some("draw".to_string())),
            chess::BoardStatus::Ongoing => (false, None),
        }
    }

    /// Similar to check_game_over but returns a Result for error handling
    /// 
    /// # Arguments
    /// * `fen` - Current board position in FEN notation
    /// 
    /// # Returns
    /// Result<(bool, Option<String>), String> - Game status or error
    pub fn check_game_status(fen: &str) -> Result<(bool, Option<String>), String> {
        let board = Board::from_str(fen).map_err(|e| format!("Invalid FEN: {}", e))?;
        
        match board.status() {
            chess::BoardStatus::Checkmate => {
                let winner = match board.side_to_move() {
                    Color::White => "black",
                    Color::Black => "white",
                };
                Ok((true, Some(winner.to_string())))
            },
            chess::BoardStatus::Stalemate => Ok((true, Some("draw".to_string()))),
            chess::BoardStatus::Ongoing => Ok((false, None)),
        }
    }

    /// Gets all legal moves for the current position
    /// 
    /// # Arguments
    /// * `fen` - Current board position in FEN notation
    /// 
    /// # Returns
    /// Result<Vec<String>, String> - List of legal moves in algebraic notation or error
    pub fn get_legal_moves(fen: &str) -> Result<Vec<String>, String> {
        let board = Board::from_str(fen).map_err(|e| format!("Invalid FEN: {}", e))?;
        
        let movegen = MoveGen::new_legal(&board);
        let moves: Vec<String> = movegen
            .map(|m| m.to_string())
            .collect();
        
        Ok(moves)
    }
}