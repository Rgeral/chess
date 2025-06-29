export interface User {
    id: string;
    username: string;
    totalGames: number;  // camelCase
    gamesWon: number;    // camelCase
    gamesLost: number;   // camelCase
    gamesDraw: number;   // camelCase
    maxDifficultyBeaten: number; // camelCase
    createdAt: string;   // camelCase
    lastPlayed?: string; // camelCase
}

export interface Game {
    id: string;
    userId: string;      // camelCase
    difficulty: number;
    fen: string;
    moves: string;
    status: string;
    startTime: string;   // camelCase
    endTime?: string;    // camelCase
    durationSeconds?: number; // camelCase
}

export interface GameMoveResult {
    game: Game;
    stockfishMove?: string;  // camelCase
    gameOver: boolean;       // camelCase
    winner?: string;
}

// ChessSquare et ChessPiece restent identiques
export interface ChessSquare {
    file: string; // a-h
    rank: number; // 1-8
    piece?: ChessPiece;
}

export interface ChessPiece {
    type: 'pawn' | 'rook' | 'knight' | 'bishop' | 'queen' | 'king';
    color: 'white' | 'black';
}