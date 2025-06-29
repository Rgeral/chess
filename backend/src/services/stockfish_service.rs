use std::process::Stdio;
use tokio::process::Command as TokioCommand;
use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader as TokioBufReader};
use rand::Rng;

/// Service for interfacing with Stockfish chess engine via UCI protocol
pub struct StockfishService;

impl StockfishService {
    /// Get best move from Stockfish with precise ELO scaling: 100-2000 ELO
    pub async fn get_best_move(fen: &str, difficulty: i32) -> Result<String, String> {
        let target_elo = difficulty * 100; // Niveau 1 = 100 ELO, Niveau 20 = 2000 ELO
        println!("🤖 Stockfish analyzing (Level {} = {}ELO): {}", difficulty, target_elo, fen);
        
        // Configuration ultra-progressive pour vraie faiblesse
        let (depth, skill_level, time_limit_ms, random_move_chance, blunder_chance) = match difficulty {
            1 => (1, -20, 1, 0.9, 0.8),       // 100 ELO - Quasi-aléatoire
            2 => (1, -18, 2, 0.8, 0.7),       // 200 ELO - Énormément d'erreurs
            3 => (1, -15, 5, 0.7, 0.6),       // 300 ELO - Beaucoup d'erreurs
            4 => (1, -12, 10, 0.6, 0.5),      // 400 ELO - Très nombreuses erreurs
            5 => (1, -10, 15, 0.5, 0.4),      // 500 ELO - Nombreuses erreurs
            6 => (1, -8, 25, 0.4, 0.3),       // 600 ELO - Erreurs fréquentes
            7 => (2, -5, 50, 0.35, 0.25),     // 700 ELO - Erreurs régulières
            8 => (2, -3, 75, 0.3, 0.2),       // 800 ELO - Pas mal d'erreurs
            9 => (2, -1, 100, 0.25, 0.15),    // 900 ELO - Quelques erreurs
            10 => (2, 0, 150, 0.2, 0.1),      // 1000 ELO - Débutant correct
            11 => (3, 2, 200, 0.15, 0.08),    // 1100 ELO - Amateur faible
            12 => (3, 4, 300, 0.12, 0.06),    // 1200 ELO - Amateur
            13 => (3, 6, 400, 0.1, 0.04),     // 1300 ELO - Amateur correct
            14 => (4, 8, 500, 0.08, 0.03),    // 1400 ELO - Amateur fort
            15 => (4, 10, 750, 0.06, 0.02),   // 1500 ELO - Club faible
            16 => (5, 12, 1000, 0.04, 0.01),  // 1600 ELO - Club
            17 => (6, 14, 1500, 0.02, 0.005), // 1700 ELO - Club fort
            18 => (7, 16, 2000, 0.01, 0.002), // 1800 ELO - Expert
            19 => (8, 18, 3000, 0.005, 0.0),  // 1900 ELO - Expert fort
            20 => (10, 20, 5000, 0.0, 0.0),   // 2000 ELO - Maître candidat
            _ => (difficulty.min(15), 20, 5000, 0.0, 0.0)
        };
        
        println!("⚙️ Settings: depth={}, skill={}, time={}ms, random={:.1}%, blunder={:.1}%", 
                 depth, skill_level, time_limit_ms, random_move_chance * 100.0, blunder_chance * 100.0);
        
        // Spawn Stockfish process
        let mut child = TokioCommand::new("stockfish")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start Stockfish: {}. Make sure Stockfish is installed.", e))?;
        
        let stdin = child.stdin.as_mut().ok_or("Failed to open stdin")?;
        let stdout = child.stdout.take().ok_or("Failed to open stdout")?;
        
        // Initialize UCI protocol
        stdin.write_all(b"uci\n").await.map_err(|e| format!("Failed to write UCI: {}", e))?;
        stdin.write_all(b"isready\n").await.map_err(|e| format!("Failed to write isready: {}", e))?;
        
        // Configure Stockfish with ultra-weak settings
        stdin.write_all(b"setoption name Skill Level value ").await
            .map_err(|e| format!("Failed to write skill option: {}", e))?;
        stdin.write_all(skill_level.to_string().as_bytes()).await
            .map_err(|e| format!("Failed to write skill value: {}", e))?;
        stdin.write_all(b"\n").await
            .map_err(|e| format!("Failed to write newline: {}", e))?;
        
        // Force ELO limit for very low levels
        if difficulty <= 10 {
            stdin.write_all(b"setoption name UCI_LimitStrength value true\n").await
                .map_err(|e| format!("Failed to set limit strength: {}", e))?;
            stdin.write_all(format!("setoption name UCI_Elo value {}\n", target_elo.max(100)).as_bytes()).await
                .map_err(|e| format!("Failed to set ELO: {}", e))?;
        }
        
        // Get multiple moves for randomness at low levels
        if difficulty <= 12 {
            stdin.write_all(b"setoption name MultiPV value 15\n").await
                .map_err(|e| format!("Failed to set MultiPV: {}", e))?;
        }
        
        stdin.write_all(b"isready\n").await.map_err(|e| format!("Failed to write isready: {}", e))?;
        
        // Set position
        stdin.write_all(format!("position fen {}\n", fen).as_bytes()).await
            .map_err(|e| format!("Failed to write position: {}", e))?;
        
        // Ultra-short time for very weak levels
        let go_command = format!("go movetime {}\n", time_limit_ms);
        stdin.write_all(go_command.as_bytes()).await
            .map_err(|e| format!("Failed to write go command: {}", e))?;
        
        // Read response and collect moves
        let mut reader = TokioBufReader::new(stdout);
        let mut line = String::new();
        let mut all_moves = Vec::new();
        let mut bad_moves = Vec::new();
        let mut random_moves = Vec::new();
        
        while reader.read_line(&mut line).await.map_err(|e| format!("Failed to read from Stockfish: {}", e))? > 0 {
            // Collect moves with their evaluations
            if line.contains("multipv") && line.contains("pv") {
                if let Some(pv_pos) = line.find("pv ") {
                    let move_part = &line[pv_pos + 3..];
                    if let Some(mv) = move_part.split_whitespace().next() {
                        if mv.len() >= 4 {
                            let score = Self::extract_score(&line).unwrap_or(0);
                            all_moves.push((mv.to_string(), score));
                            
                            // Classify moves for different weakness levels
                            if score < -300 {
                                bad_moves.push(mv.to_string()); // Blunders
                            } else if score < 0 {
                                random_moves.push(mv.to_string()); // Weak moves
                            }
                        }
                    }
                }
            }
            
            if line.starts_with("bestmove") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let stockfish_best = parts[1].to_string();
                    
                    // Apply weakness based on precise difficulty level
                    let final_move = Self::apply_weakness(
                        stockfish_best,
                        &all_moves,
                        &bad_moves,
                        &random_moves,
                        random_move_chance,
                        blunder_chance,
                        difficulty
                    );
                    
                    // Terminate Stockfish
                    let _ = stdin.write_all(b"quit\n").await;
                    let _ = child.wait().await;
                    
                    println!("✅ Final move chosen: {} (target: {}ELO)", final_move, target_elo);
                    return Ok(final_move);
                }
            }
            line.clear();
        }
        
        Err("No move returned by Stockfish".to_string())
    }
    
    /// Apply weakness to move selection based on difficulty
    fn apply_weakness(
        best_move: String,
        all_moves: &[(String, i32)],
        bad_moves: &[String],
        random_moves: &[String],
        random_chance: f64,
        blunder_chance: f64,
        difficulty: i32
    ) -> String {
        let mut rng = rand::thread_rng();
        
        // Ultra-low levels: High chance of terrible moves
        if difficulty <= 5 && !bad_moves.is_empty() && rng.gen::<f64>() < blunder_chance {
            let random_bad = &bad_moves[rng.gen_range(0..bad_moves.len())];
            println!("🤦 Chose blunder move for level {}", difficulty);
            return random_bad.clone();
        }
        
        // Low levels: Often pick random moves
        if difficulty <= 10 && !all_moves.is_empty() && rng.gen::<f64>() < random_chance {
            // Pick from bottom half of moves
            let bottom_half_start = all_moves.len() / 2;
            if bottom_half_start < all_moves.len() {
                let random_index = rng.gen_range(bottom_half_start..all_moves.len());
                let random_move = &all_moves[random_index].0;
                println!("🎲 Chose random weak move for level {}", difficulty);
                return random_move.clone();
            }
        }
        
        // Medium levels: Sometimes pick suboptimal moves
        if difficulty <= 15 && all_moves.len() > 3 && rng.gen::<f64>() < random_chance {
            // Pick from top 60% of moves (avoiding the very best)
            let range_end = (all_moves.len() as f64 * 0.6).max(3.0) as usize;
            let random_index = rng.gen_range(1..range_end.min(all_moves.len()));
            let suboptimal_move = &all_moves[random_index].0;
            println!("🤔 Chose suboptimal move for level {}", difficulty);
            return suboptimal_move.clone();
        }
        
        // Default: return best move
        best_move
    }
    
    /// Extract centipawn score from Stockfish output
    fn extract_score(line: &str) -> Option<i32> {
        if let Some(cp_pos) = line.find("score cp ") {
            let score_part = &line[cp_pos + 9..];
            if let Some(score_str) = score_part.split_whitespace().next() {
                return score_str.parse().ok();
            }
        }
        None
    }
    
    /// Evaluate position (returns centipawn score)
    pub async fn evaluate_position(fen: &str, depth: i32) -> Result<i32, String> {
        println!("📊 Evaluating position: {}", fen);
        
        let mut child = TokioCommand::new("stockfish")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start Stockfish: {}", e))?;
        
        let stdin = child.stdin.as_mut().ok_or("Failed to open stdin")?;
        let stdout = child.stdout.take().ok_or("Failed to open stdout")?;
        
        stdin.write_all(b"uci\n").await.map_err(|e| format!("Failed to write: {}", e))?;
        stdin.write_all(b"isready\n").await.map_err(|e| format!("Failed to write: {}", e))?;
        stdin.write_all(format!("position fen {}\n", fen).as_bytes()).await.map_err(|e| format!("Failed to write: {}", e))?;
        stdin.write_all(format!("go depth {}\n", depth).as_bytes()).await.map_err(|e| format!("Failed to write: {}", e))?;
        
        let mut reader = TokioBufReader::new(stdout);
        let mut line = String::new();
        let mut score = 0;
        
        while reader.read_line(&mut line).await.map_err(|e| format!("Failed to read: {}", e))? > 0 {
            if let Some(cp_score) = Self::extract_score(&line) {
                score = cp_score;
            }
            if line.starts_with("bestmove") {
                break;
            }
            line.clear();
        }
        
        let _ = stdin.write_all(b"quit\n").await;
        let _ = child.wait().await;
        
        Ok(score)
    }
}