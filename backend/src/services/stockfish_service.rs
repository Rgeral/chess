use std::process::Stdio;
use tokio::process::Command as TokioCommand;
use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader as TokioBufReader};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use tracing::{debug, error, info, warn};
use tokio::time::{Duration, Instant};

/// Service for interfacing with Stockfish chess engine via UCI protocol
pub struct StockfishService;

impl StockfishService {
    /// Returns the path/command to Stockfish (env override then common paths)
    fn get_stockfish_command() -> String {
        if let Ok(cmd) = std::env::var("STOCKFISH_PATH") {
            if std::process::Command::new(&cmd).arg("--help").output().is_ok() {
                info!("Using Stockfish from env: {}", cmd);
                return cmd;
            } else {
                warn!("STOCKFISH_PATH provided but not executable: {}", cmd);
            }
        }
        let possible_paths = vec![
            "stockfish",
            "/usr/games/stockfish",
            "/usr/local/bin/stockfish",
            "/usr/bin/stockfish",
        ];
        for path in possible_paths {
            if std::process::Command::new(path).arg("--help").output().is_ok() {
                info!("Stockfish found at: {}", path);
                return path.to_string();
            }
        }
        warn!("Stockfish not found, using default command name");
        "stockfish".to_string()
    }

    /// Returns the best move for a FEN at a given difficulty (with intentional weakness)
    pub async fn get_best_move(fen: &str, difficulty: i32) -> Result<String, String> {
        let target_elo = difficulty * 100;
        debug!("Analyze: level={} elo={} fen={}", difficulty, target_elo, fen);

        let (skill_level, time_limit_ms, random_move_chance, blunder_chance) = match difficulty {
            1 => (-20, 1, 0.95, 0.9),
            2 => (-18, 1, 0.9, 0.8),
            3 => (-15, 2, 0.85, 0.7),
            4 => (-12, 5, 0.8, 0.6),
            5 => (-10, 10, 0.75, 0.5),
            6 => (-8, 25, 0.4, 0.3),
            7 => (-5, 50, 0.35, 0.25),
            8 => (-3, 75, 0.3, 0.2),
            9 => (-1, 100, 0.25, 0.15),
            10 => (0, 150, 0.2, 0.1),
            11 => (2, 200, 0.15, 0.08),
            12 => (4, 300, 0.12, 0.06),
            13 => (6, 400, 0.1, 0.04),
            14 => (8, 500, 0.08, 0.03),
            15 => (10, 750, 0.06, 0.02),
            16 => (12, 450, 0.04, 0.01),
            17 => (14, 350, 0.02, 0.005),
            18 => (16, 300, 0.01, 0.002),
            19 => (18, 250, 0.005, 0.0),
            20 => (20, 200, 0.003, 0.0),
            _ => (20, 200, 0.0, 0.0),
        };

        debug!(
            "Settings: skill={} time={}ms random={:.1}% blunder={:.1}%",
            skill_level,
            time_limit_ms,
            random_move_chance * 100.0,
            blunder_chance * 100.0
        );

        if difficulty <= 5 {
            let mut rng = StdRng::from_entropy();
            let random_roll = rng.gen::<f64>();
            debug!("Random roll: level={} roll={:.3} threshold={:.3}", difficulty, random_roll, random_move_chance);
            if random_roll < random_move_chance {
                debug!("Generating random legal move (low level)");
                match Self::get_random_legal_move(fen).await {
                    Ok(random_move) => return Ok(random_move),
                    Err(e) => warn!("Random move generation failed: {}", e),
                }
            }
        }

        let stockfish_move = Self::get_stockfish_move_with_weakness(
            fen,
            skill_level,
            time_limit_ms,
            random_move_chance,
            blunder_chance,
            difficulty,
        )
        .await?;

        Ok(stockfish_move)
    }

    /// Generates a completely random legal move using the chess crate (low levels)
    async fn get_random_legal_move(fen: &str) -> Result<String, String> {
        use chess::{Board, MoveGen};
        use std::str::FromStr;
        let board = Board::from_str(fen).map_err(|e| format!("Invalid FEN: {}", e))?;
        let movegen = MoveGen::new_legal(&board);
        let legal_moves: Vec<String> = movegen.map(|m| m.to_string()).collect();
        debug!("{} legal moves available for random selection", legal_moves.len());
        if legal_moves.is_empty() {
            return Err("No legal moves found".to_string());
        }
        let mut rng = StdRng::from_entropy();
        let chosen_move = legal_moves[rng.gen_range(0..legal_moves.len())].clone();
        Ok(chosen_move)
    }

    /// Computes a move with Stockfish then applies weakness according to difficulty
    async fn get_stockfish_move_with_weakness(
        fen: &str,
        skill_level: i32,
        time_limit_ms: i32,
        random_move_chance: f64,
        blunder_chance: f64,
        difficulty: i32,
    ) -> Result<String, String> {
        let stockfish_cmd = Self::get_stockfish_command();
        let mut child = TokioCommand::new(&stockfish_cmd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start Stockfish: {}", e))?;

        let mut stdin = child.stdin.take().ok_or("Failed to open stdin")?;
        let stdout = child.stdout.take().ok_or("Failed to open stdout")?;

        // Initialize UCI and configure options
        Self::uci_init(&mut stdin).await?;
        Self::configure_engine(&mut stdin, skill_level, difficulty).await?;
        Self::set_position_and_go(&mut stdin, fen, time_limit_ms).await?;

        let mut reader = TokioBufReader::new(stdout);
        let (best, all_moves, bad_moves) = Self::collect_moves_with_timeout(
            &mut reader,
            time_limit_ms,
        )
        .await?;

        // If engine provided no explicit bestmove (timeout), fallback to best from list or random
        let best_move = best.or_else(|| {
            all_moves
                .iter()
                .max_by_key(|(_, s)| *s)
                .map(|(m, _)| m.clone())
        });

        let final_move = if let Some(bm) = best_move {
            Self::apply_weakness(
                bm,
                &all_moves,
                &bad_moves,
                random_move_chance,
                blunder_chance,
                difficulty,
            )
        } else {
            // Last resort: generate a random legal move quickly
            match Self::get_random_legal_move(fen).await {
                Ok(mv) => mv,
                Err(_) => {
                    // If everything fails, terminate and error
                    let _ = stdin.write_all(b"quit\n").await;
                    let _ = child.kill().await;
                    return Err("Engine timeout without moves".to_string());
                }
            }
        };

        // Shutdown engine
        let _ = stdin.write_all(b"quit\n").await;
        let _ = child.wait().await;
        debug!("Final move: {} (level={} elo={})", final_move, difficulty, difficulty * 100);
        Ok(final_move)
    }

    /// Applies weakness to move selection (kept separate for testability)
    fn apply_weakness(
        best_move: String,
        all_moves: &[(String, i32)],
        bad_moves: &[String],
        random_chance: f64,
        blunder_chance: f64,
        difficulty: i32,
    ) -> String {
        let mut rng = StdRng::from_entropy();
        if difficulty <= 3 && !bad_moves.is_empty() && rng.gen::<f64>() < blunder_chance {
            return bad_moves[rng.gen_range(0..bad_moves.len())].clone();
        }
        if difficulty <= 5 && !all_moves.is_empty() && rng.gen::<f64>() < random_chance {
            return all_moves[rng.gen_range(0..all_moves.len())].0.clone();
        }
        if difficulty <= 10 && !all_moves.is_empty() && rng.gen::<f64>() < random_chance {
            let bottom_start = (all_moves.len() as f64 * 0.3) as usize;
            if bottom_start < all_moves.len() {
                return all_moves[rng.gen_range(bottom_start..all_moves.len())].0.clone();
            }
        }
        if difficulty <= 15 && all_moves.len() > 3 && rng.gen::<f64>() < random_chance {
            let range_end = (all_moves.len() as f64 * 0.6).max(3.0) as usize;
            return all_moves[rng.gen_range(1..range_end.min(all_moves.len()))].0.clone();
        }
        best_move
    }

    /// Extracts centipawn score from a single Stockfish info line
    fn extract_score(line: &str) -> Option<i32> {
        if let Some(cp_pos) = line.find("score cp ") {
            let score_part = &line[cp_pos + 9..];
            if let Some(score_str) = score_part.split_whitespace().next() {
                return score_str.parse().ok();
            }
        }
        None
    }

    /// Evaluates a position (centipawn score) with a limited depth
    pub async fn evaluate_position(fen: &str, depth: i32) -> Result<i32, String> {
        debug!("Evaluate fen={} depth={}", fen, depth);
        let stockfish_cmd = Self::get_stockfish_command();
        let mut child = TokioCommand::new(&stockfish_cmd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start Stockfish: {}", e))?;
        let mut stdin = child.stdin.take().ok_or("Failed to open stdin")?;
        let stdout = child.stdout.take().ok_or("Failed to open stdout")?;
        // UCI init & position
        Self::uci_init(&mut stdin).await?;
        stdin
            .write_all(format!("position fen {}\n", fen).as_bytes())
            .await
            .map_err(|e| format!("Failed to write position: {}", e))?;
        stdin
            .write_all(format!("go depth {}\n", depth).as_bytes())
            .await
            .map_err(|e| format!("Failed to write go: {}", e))?;

        let mut reader = TokioBufReader::new(stdout);
        let mut line = String::new();
        let mut score = 0;
        let start = Instant::now();
        let max = Duration::from_secs(2);
        loop {
            let remaining = max.checked_sub(start.elapsed()).unwrap_or(Duration::from_millis(0));
            if remaining.is_zero() {
                warn!("Evaluation timeout");
                break;
            }
            match tokio::time::timeout(remaining, reader.read_line(&mut line)).await {
                Ok(Ok(n)) if n > 0 => {
                    if let Some(cp) = Self::extract_score(&line) { score = cp; }
                    if line.starts_with("bestmove") { break; }
                    line.clear();
                }
                Ok(Ok(_)) => break,
                Ok(Err(e)) => return Err(format!("Failed to read: {}", e)),
                Err(_) => { warn!("Evaluation read timeout"); break; }
            }
        }
        let _ = stdin.write_all(b"quit\n").await;
        let _ = child.wait().await;
        Ok(score)
    }

    // --- helpers (no verbose comments inline) ---
    async fn uci_init(stdin: &mut tokio::process::ChildStdin) -> Result<(), String> {
        stdin.write_all(b"uci\n").await.map_err(|e| format!("Failed to write UCI: {}", e))?;
        stdin.write_all(b"isready\n").await.map_err(|e| format!("Failed to write isready: {}", e))?;
        Ok(())
    }

    async fn configure_engine(
        stdin: &mut tokio::process::ChildStdin,
        skill_level: i32,
        difficulty: i32,
    ) -> Result<(), String> {
        stdin
            .write_all(format!("setoption name Skill Level value {}\n", skill_level).as_bytes())
            .await
            .map_err(|e| format!("Failed to set skill: {}", e))?;
        if difficulty <= 10 {
            let target_elo = (difficulty * 100).max(100);
            stdin
                .write_all(b"setoption name UCI_LimitStrength value true\n")
                .await
                .map_err(|e| format!("Failed to set limit strength: {}", e))?;
            stdin
                .write_all(format!("setoption name UCI_Elo value {}\n", target_elo).as_bytes())
                .await
                .map_err(|e| format!("Failed to set ELO: {}", e))?;
        }
        if difficulty <= 15 {
            stdin
                .write_all(b"setoption name MultiPV value 20\n")
                .await
                .map_err(|e| format!("Failed to set MultiPV: {}", e))?;
        }
        stdin.write_all(b"isready\n").await.map_err(|e| format!("Failed to write isready: {}", e))?;
        Ok(())
    }

    async fn set_position_and_go(
        stdin: &mut tokio::process::ChildStdin,
        fen: &str,
        time_limit_ms: i32,
    ) -> Result<(), String> {
        stdin
            .write_all(format!("position fen {}\n", fen).as_bytes())
            .await
            .map_err(|e| format!("Failed to write position: {}", e))?;
        stdin
            .write_all(format!("go movetime {}\n", time_limit_ms).as_bytes())
            .await
            .map_err(|e| format!("Failed to write go command: {}", e))?;
        Ok(())
    }

    async fn collect_moves_with_timeout(
        reader: &mut TokioBufReader<tokio::process::ChildStdout>,
        time_limit_ms: i32,
    ) -> Result<(Option<String>, Vec<(String, i32)>, Vec<String>), String> {
        let mut line = String::new();
        let mut all_moves: Vec<(String, i32)> = Vec::new();
        let mut bad_moves: Vec<String> = Vec::new();
        let mut best: Option<String> = None;
        let start = Instant::now();
        let max = Duration::from_millis(time_limit_ms as u64 + 400);
        loop {
            let remaining = max.checked_sub(start.elapsed()).unwrap_or(Duration::from_millis(0));
            if remaining.is_zero() {
                warn!("Stockfish read timeout reached");
                break;
            }
            match tokio::time::timeout(remaining, reader.read_line(&mut line)).await {
                Ok(Ok(n)) if n > 0 => {
                    if line.contains("multipv") && line.contains("pv") {
                        if let Some(pv_pos) = line.find("pv ") {
                            let move_part = &line[pv_pos + 3..];
                            if let Some(mv) = move_part.split_whitespace().next() {
                                if mv.len() >= 4 {
                                    let score = Self::extract_score(&line).unwrap_or(0);
                                    all_moves.push((mv.to_string(), score));
                                    if score < -200 { bad_moves.push(mv.to_string()); }
                                }
                            }
                        }
                    }
                    if line.starts_with("bestmove") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 { best = Some(parts[1].to_string()); }
                        break;
                    }
                    line.clear();
                }
                Ok(Ok(_)) => break,
                Ok(Err(e)) => return Err(format!("Failed to read from Stockfish: {}", e)),
                Err(_) => { warn!("Per-line read timeout"); break; }
            }
        }
        Ok((best, all_moves, bad_moves))
    }
}