use std::process::Stdio;
use tokio::process::Command as TokioCommand;
use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader as TokioBufReader};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

/// Service for interfacing with Stockfish chess engine via UCI protocol
pub struct StockfishService;

impl StockfishService {
    /// Obtient la commande Stockfish appropriée selon l'environnement
    fn get_stockfish_command() -> String {
        // Essayer différents chemins possibles pour Stockfish
        let possible_paths = vec![
            "stockfish",           // Dans le PATH
            "/usr/games/stockfish", // Installation Ubuntu/Debian standard
            "/usr/local/bin/stockfish", // Installation via lien symbolique
            "/usr/bin/stockfish",  // Installation alternative
        ];
        
        for path in possible_paths {
            if std::process::Command::new(path)
                .arg("--help")
                .output()
                .is_ok() {
                println!("🔍 Stockfish trouvé à: {}", path);
                return path.to_string();
            }
        }
        
        println!("⚠️  Stockfish non trouvé, utilisation du chemin par défaut");
        "stockfish".to_string()
    }

    /// Get best move from Stockfish with VRAIE faiblesse pour niveaux 1-5
    pub async fn get_best_move(fen: &str, difficulty: i32) -> Result<String, String> {
        let target_elo = difficulty * 100;
        println!("🤖 Stockfish analyzing (Level {} = {}ELO): {}", difficulty, target_elo, fen);
        
        // Configuration ULTRA-AGGRESSIVE pour vraie faiblesse
        let (skill_level, time_limit_ms, random_move_chance, blunder_chance) = match difficulty {
            1 => (-20, 1, 0.95, 0.9),          // 100 ELO - PRESQUE toujours aléatoire
            2 => (-18, 1, 0.9, 0.8),           // 200 ELO - Très souvent aléatoire  
            3 => (-15, 2, 0.85, 0.7),          // 300 ELO - Souvent aléatoire
            4 => (-12, 5, 0.8, 0.6),           // 400 ELO - Assez souvent aléatoire
            5 => (-10, 10, 0.75, 0.5),         // 500 ELO - Régulièrement aléatoire
            6 => (-8, 25, 0.4, 0.3),           // 600 ELO - Erreurs fréquentes
            7 => (-5, 50, 0.35, 0.25),         // 700 ELO - Erreurs régulières
            8 => (-3, 75, 0.3, 0.2),           // 800 ELO - Pas mal d'erreurs
            9 => (-1, 100, 0.25, 0.15),        // 900 ELO - Quelques erreurs
            10 => (0, 150, 0.2, 0.1),          // 1000 ELO - Débutant correct
            11 => (2, 200, 0.15, 0.08),        // 1100 ELO - Amateur faible
            12 => (4, 300, 0.12, 0.06),        // 1200 ELO - Amateur
            13 => (6, 400, 0.1, 0.04),         // 1300 ELO - Amateur correct
            14 => (8, 500, 0.08, 0.03),        // 1400 ELO - Amateur fort
            15 => (10, 750, 0.06, 0.02),       // 1500 ELO - Club faible
            16 => (12, 1000, 0.04, 0.01),      // 1600 ELO - Club
            17 => (14, 1500, 0.02, 0.005),     // 1700 ELO - Club fort
            18 => (16, 2000, 0.01, 0.002),     // 1800 ELO - Expert
            19 => (18, 3000, 0.005, 0.0),      // 1900 ELO - Expert fort
            20 => (20, 5000, 0.0, 0.0),        // 2000 ELO - Maître candidat
            _ => (20, 5000, 0.0, 0.0)
        };
        
        println!("⚙️ Settings: skill={}, time={}ms, random={:.1}%, blunder={:.1}%", 
                 skill_level, time_limit_ms, random_move_chance * 100.0, blunder_chance * 100.0);

        // NOUVEAU: Pour les niveaux 1-5, on force l'aléatoire AVANT même Stockfish
        if difficulty <= 5 {
            // Créer un RNG thread-safe
            let mut rng = StdRng::from_entropy();
            if rng.gen::<f64>() < random_move_chance {
                // Générer directement un coup aléatoire legal
                if let Ok(random_move) = Self::get_random_legal_move(fen).await {
                    println!("🎲 NIVEAU {}: Coup complètement aléatoire choisi: {}", difficulty, random_move);
                    return Ok(random_move);
                }
            }
        }
        
        // Si pas d'aléatoire forcé, continuer avec Stockfish ultra-faible
        let stockfish_move = Self::get_stockfish_move_with_weakness(
            fen, skill_level, time_limit_ms, random_move_chance, blunder_chance, difficulty
        ).await?;
        
        Ok(stockfish_move)
    }

    /// Génère un coup légal complètement aléatoire (pour niveaux 1-5)
    async fn get_random_legal_move(fen: &str) -> Result<String, String> {
        let stockfish_cmd = Self::get_stockfish_command();
        let mut child = TokioCommand::new(&stockfish_cmd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start Stockfish: {}", e))?;
        
        let stdin = child.stdin.as_mut().ok_or("Failed to open stdin")?;
        let stdout = child.stdout.take().ok_or("Failed to open stdout")?;
        
        // Init rapide
        stdin.write_all(b"uci\n").await.map_err(|e| format!("Failed to write: {}", e))?;
        stdin.write_all(b"isready\n").await.map_err(|e| format!("Failed to write: {}", e))?;
        stdin.write_all(format!("position fen {}\n", fen).as_bytes()).await.map_err(|e| format!("Failed to write: {}", e))?;
        
        // Demander TOUS les coups légaux avec MultiPV élevé
        stdin.write_all(b"setoption name MultiPV value 50\n").await.map_err(|e| format!("Failed to write: {}", e))?;
        stdin.write_all(b"go movetime 10\n").await.map_err(|e| format!("Failed to write: {}", e))?;
        
        let mut reader = TokioBufReader::new(stdout);
        let mut line = String::new();
        let mut legal_moves = Vec::new();
        
        while reader.read_line(&mut line).await.map_err(|e| format!("Failed to read: {}", e))? > 0 {
            if line.contains("multipv") && line.contains("pv") {
                if let Some(pv_pos) = line.find("pv ") {
                    let move_part = &line[pv_pos + 3..];
                    if let Some(mv) = move_part.split_whitespace().next() {
                        if mv.len() >= 4 {
                            legal_moves.push(mv.to_string());
                        }
                    }
                }
            }
            if line.starts_with("bestmove") {
                break;
            }
            line.clear();
        }
        
        let _ = stdin.write_all(b"quit\n").await;
        let _ = child.wait().await;
        
        // Choisir un coup aléatoire parmi TOUS les coups légaux
        if !legal_moves.is_empty() {
            let mut rng = StdRng::from_entropy();
            let random_index = rng.gen_range(0..legal_moves.len());
            Ok(legal_moves[random_index].clone())
        } else {
            Err("No legal moves found".to_string())
        }
    }

    /// Version Stockfish avec faiblesse appliquée
    async fn get_stockfish_move_with_weakness(
        fen: &str,
        skill_level: i32,
        time_limit_ms: i32,
        random_move_chance: f64,
        blunder_chance: f64,
        difficulty: i32
    ) -> Result<String, String> {
        // Spawn Stockfish process
        let stockfish_cmd = Self::get_stockfish_command();
        let mut child = TokioCommand::new(&stockfish_cmd)
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
        
        // Configure Stockfish avec settings ultra-faibles
        stdin.write_all(format!("setoption name Skill Level value {}\n", skill_level).as_bytes()).await
            .map_err(|e| format!("Failed to write skill: {}", e))?;
        
        // Force ELO limit pour niveaux faibles
        if difficulty <= 10 {
            let target_elo = (difficulty * 100).max(100);
            stdin.write_all(b"setoption name UCI_LimitStrength value true\n").await
                .map_err(|e| format!("Failed to set limit strength: {}", e))?;
            stdin.write_all(format!("setoption name UCI_Elo value {}\n", target_elo).as_bytes()).await
                .map_err(|e| format!("Failed to set ELO: {}", e))?;
        }
        
        // MultiPV pour avoir plusieurs options
        if difficulty <= 15 {
            stdin.write_all(b"setoption name MultiPV value 20\n").await
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
                            if score < -200 {
                                bad_moves.push(mv.to_string()); // Blunders
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
                        random_move_chance,
                        blunder_chance,
                        difficulty
                    );
                    
                    // Terminate Stockfish
                    let _ = stdin.write_all(b"quit\n").await;
                    let _ = child.wait().await;
                    
                    println!("✅ Final move chosen: {} (target: {}ELO)", final_move, difficulty * 100);
                    return Ok(final_move);
                }
            }
            line.clear();
        }
        
        Err("No move returned by Stockfish".to_string())
    }

    /// Apply weakness to move selection - VERSION RENFORCÉE
    fn apply_weakness(
        best_move: String,
        all_moves: &[(String, i32)],
        bad_moves: &[String],
        random_chance: f64,
        blunder_chance: f64,
        difficulty: i32
    ) -> String {
        let mut rng = StdRng::from_entropy();
        
        // NIVEAUX 1-3: Priorité aux VRAIMENT mauvais coups
        if difficulty <= 3 && !bad_moves.is_empty() && rng.gen::<f64>() < blunder_chance {
            let random_bad = &bad_moves[rng.gen_range(0..bad_moves.len())];
            println!("🤦 Niveau {}: Blunder choisi: {}", difficulty, random_bad);
            return random_bad.clone();
        }
        
        // NIVEAUX 1-5: Très souvent des coups aléatoires
        if difficulty <= 5 && !all_moves.is_empty() && rng.gen::<f64>() < random_chance {
            // Pour les niveaux ultra-faibles, prendre vraiment n'importe quoi
            let random_index = rng.gen_range(0..all_moves.len());
            let random_move = &all_moves[random_index].0;
            println!("🎲 Niveau {}: Coup complètement aléatoire: {}", difficulty, random_move);
            return random_move.clone();
        }
        
        // NIVEAUX 6-10: Souvent des coups faibles
        if difficulty <= 10 && !all_moves.is_empty() && rng.gen::<f64>() < random_chance {
            // Pick from bottom 70% of moves
            let bottom_start = (all_moves.len() as f64 * 0.3) as usize;
            if bottom_start < all_moves.len() {
                let random_index = rng.gen_range(bottom_start..all_moves.len());
                let weak_move = &all_moves[random_index].0;
                println!("🤔 Niveau {}: Coup faible: {}", difficulty, weak_move);
                return weak_move.clone();
            }
        }
        
        // NIVEAUX 11-15: Parfois des coups suboptimaux
        if difficulty <= 15 && all_moves.len() > 3 && rng.gen::<f64>() < random_chance {
            // Pick from top 60% of moves (avoiding the very best)
            let range_end = (all_moves.len() as f64 * 0.6).max(3.0) as usize;
            let random_index = rng.gen_range(1..range_end.min(all_moves.len()));
            let suboptimal_move = &all_moves[random_index].0;
            println!("💭 Niveau {}: Coup suboptimal: {}", difficulty, suboptimal_move);
            return suboptimal_move.clone();
        }
        
        // Default: return best move
        println!("🎯 Niveau {}: Meilleur coup: {}", difficulty, best_move);
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
        
        let stockfish_cmd = Self::get_stockfish_command();
        let mut child = TokioCommand::new(&stockfish_cmd)
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