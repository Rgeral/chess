-- Users table
CREATE TABLE users (
    id TEXT PRIMARY KEY NOT NULL,
    username TEXT NOT NULL UNIQUE,
    total_games INTEGER NOT NULL DEFAULT 0,
    games_won INTEGER NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    total_play_time_seconds INTEGER DEFAULT 0,
    current_streak INTEGER DEFAULT 0,
    best_streak INTEGER DEFAULT 0,
    estimated_elo INTEGER DEFAULT 800
);

-- Games table
CREATE TABLE games (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    difficulty INTEGER NOT NULL,
    fen TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    result TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    start_time DATETIME,
    end_time DATETIME,
    duration_seconds INTEGER,
    moves_count INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- User level statistics
CREATE TABLE user_level_stats (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    difficulty INTEGER NOT NULL,
    games_played INTEGER NOT NULL DEFAULT 0,
    games_won INTEGER NOT NULL DEFAULT 0,
    total_time_seconds INTEGER NOT NULL DEFAULT 0,
    average_time_seconds INTEGER NOT NULL DEFAULT 0,
    total_moves INTEGER NOT NULL DEFAULT 0,
    average_moves INTEGER NOT NULL DEFAULT 0,
    UNIQUE(user_id, difficulty),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- User personal records
CREATE TABLE user_records (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    difficulty INTEGER NOT NULL,
    best_time_seconds INTEGER NOT NULL,
    moves_count INTEGER NOT NULL,
    achieved_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, difficulty),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Indexes for performance
CREATE INDEX idx_games_user_id ON games(user_id);
CREATE INDEX idx_games_status ON games(status);
CREATE INDEX idx_user_level_stats_user_id ON user_level_stats(user_id);
CREATE INDEX idx_user_records_user_id ON user_records(user_id);