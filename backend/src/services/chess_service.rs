use chess::{Board, ChessMove, Color, MoveGen, Square, Piece};
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

	/// Determines the en passant target square after a pawn's double move
	/// 
	/// # Arguments
	/// * `from` - Starting square of the pawn
	/// * `to` - Ending square of the pawn
	/// * `piece` - Piece being moved
	/// 
	/// # Returns
	/// Option<String> - En passant target square in algebraic notation or None
	fn en_passant_target(from: &Square, to: &Square, piece: Option<Piece>) -> Option<String> {
        use std::str::FromStr;
        if piece != Some(Piece::Pawn) {
            return None;
        }
        let from_s = format!("{}", from); // ex "g7"
        let to_s = format!("{}", to);     // ex "g5"
        let from_rank = from_s.chars().nth(1)?.to_digit(10)? as i32;
        let to_rank = to_s.chars().nth(1)?.to_digit(10)? as i32;
        if (from_rank - to_rank).abs() == 2 {
            let file = from_s.chars().next()?;
            let ep_rank = (from_rank + to_rank) / 2;
            return Some(format!("{}{}", file, ep_rank));
        }
        None
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
        
        println!("🎯 Processing move: {}", move_str);
        
        // Parse the move - handle promotion
        let chess_move = if move_str.len() == 5 {
            // Promotion move (e.g., "e7e8q")
            let from_square = Square::from_str(&move_str[0..2])
                .map_err(|_| format!("Invalid source square: {}", &move_str[0..2]))?;
            let to_square = Square::from_str(&move_str[2..4])
                .map_err(|_| format!("Invalid target square: {}", &move_str[2..4]))?;
            
            let promotion_char = move_str.chars().nth(4).unwrap();
            let promotion_piece = match promotion_char {
                'q' => Some(Piece::Queen),
                'r' => Some(Piece::Rook),
                'b' => Some(Piece::Bishop),
                'n' => Some(Piece::Knight),
                _ => return Err(format!("Invalid promotion piece: {}", promotion_char))
            };
            
            println!("🎯 Promotion detected: {} to {} promoting to {:?}", 
                     from_square, to_square, promotion_piece);
            
            ChessMove::new(from_square, to_square, promotion_piece)
        } else {
            // Regular move (e.g., "e2e4")
            ChessMove::from_str(move_str)
                .map_err(|e| format!("Invalid move format: {}", e))?
        };
        
        // if !board.legal(chess_move) {
        //     return Err(format!("Illegal move: {}", move_str));
        // }

		// Extract source/dest squares from the ChessMove
         let from_square = chess_move.get_source();
		let to_square = chess_move.get_dest();

		// Check legality without consuming the ChessMove
		let legal = MoveGen::new_legal(&board).any(|m| m == chess_move);
		if !legal {
			return Err(format!("Illegal move: {}", move_str));
		}

		// Determine moved piece before applying the move
        let moved_piece = board.piece_on(from_square);
        
        board = board.make_move_new(chess_move);
        println!("✅ Move applied successfully");

		// Build FEN and ensure en-passant target is correct for double pawn-step
        let mut fen_after = board.to_string();
        if let Some(ep) = Self::en_passant_target(&from_square, &to_square, moved_piece) {
            // replace the 4th field (en-passant) with our computed target if it's different
            let mut parts: Vec<String> = fen_after.split_whitespace().map(|s| s.to_string()).collect();
            if parts.len() == 6 && parts[3] != ep {
                parts[3] = ep;
                fen_after = parts.join(" ");
            }
        }

        Ok(fen_after)
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

    /// Gets the piece type at a specific square
    /// 
    /// # Arguments
    /// * `fen` - Current board position in FEN notation
    /// * `square_notation` - Square in algebraic notation (e.g., "e4")
    /// 
    /// # Returns
    /// Option<String> - Piece type name or None if empty
    pub fn get_piece_at_square(fen: &str, square_notation: &str) -> Option<String> {
        let board = Board::from_str(fen).ok()?;
        let square = Square::from_str(square_notation).ok()?;
        
        if let Some(piece) = board.piece_on(square) {
            let piece_name = match piece {
                chess::Piece::Pawn => "pawn",
                chess::Piece::Knight => "knight", 
                chess::Piece::Bishop => "bishop",
                chess::Piece::Rook => "rook",
                chess::Piece::Queen => "queen",
                chess::Piece::King => "king",
            };
            Some(piece_name.to_string())
        } else {
            None
        }
    }
}