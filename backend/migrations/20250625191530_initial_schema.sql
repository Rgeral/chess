CREATE TABLE users (
    id TEXT PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    total_games INTEGER DEFAULT 0,
    games_won INTEGER DEFAULT 0,
    games_lost INTEGER DEFAULT 0,
    games_draw INTEGER DEFAULT 0,
    max_difficulty_beaten INTEGER DEFAULT 0,
    last_played TIMESTAMP
);

CREATE TABLE user_best_times (
    user_id TEXT NOT NULL,
    difficulty INTEGER NOT NULL,
    best_time_seconds INTEGER NOT NULL,
    achieved_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, difficulty),
    FOREIGN KEY (user_id) REFERENCES users (id)
);

CREATE TABLE games (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    difficulty INTEGER NOT NULL, -- 1-20 (profondeur Stockfish)
    fen TEXT NOT NULL,           -- Position finale
    moves TEXT NOT NULL,         -- JSON array des coups joués
    status TEXT NOT NULL,        -- 'won', 'lost', 'draw'
    start_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    end_time TIMESTAMP,
    duration_seconds INTEGER,
    FOREIGN KEY (user_id) REFERENCES users (id)
);

CREATE TABLE scores (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    game_id TEXT NOT NULL,
    difficulty INTEGER NOT NULL,
    duration_seconds INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users (id),
    FOREIGN KEY (game_id) REFERENCES games (id)
);

CREATE INDEX idx_users_username ON users (username);
CREATE INDEX idx_scores_difficulty_duration ON scores (difficulty, duration_seconds);
CREATE INDEX idx_games_user_id ON games (user_id);
CREATE INDEX idx_best_times_difficulty ON user_best_times (difficulty, best_time_seconds);