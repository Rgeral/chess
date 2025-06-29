export interface User {
    id: string;
    username: string;
    totalGames: number;
    gamesWon: number;
    createdAt: string;
    totalPlayTimeSeconds?: number;
    currentStreak?: number;
    bestStreak?: number;
    estimatedElo?: number;
}

export interface Game {
    id: string;
    userId: string;
    difficulty: number;
    fen: string;
    status: string;
    result?: string;
    createdAt: string;
    startTime?: string;
    endTime?: string;
    durationSeconds?: number;
    movesCount: number;
}

export interface UserRecord {
    id: string;
    userId: string;
    difficulty: number;
    bestTimeSeconds: number;
    movesCount: number;
    achievedAt?: string;
}

export interface UserLevelStats {
    id: string;
    userId: string;
    difficulty: number;
    gamesPlayed: number;
    gamesWon: number;
    totalTimeSeconds: number;
    averageTimeSeconds: number;
    totalMoves: number;
    averageMoves: number;
}

export interface UserProfile {
    user: User;
    records: UserRecord[];
    levelStats: UserLevelStats[];
}

export interface GameMoveResult {
    game: Game;
    stockfishMove: string;
    gameOver: boolean;
    winner?: string;
    moveTimeMs?: number;
    totalTimeSeconds?: number;
}

export interface ChessPiece {
    type: 'king' | 'queen' | 'rook' | 'bishop' | 'knight' | 'pawn';
    color: 'white' | 'black';
}

export interface ChessSquare {
    file: string;
    rank: number;
    piece: ChessPiece | null;
}