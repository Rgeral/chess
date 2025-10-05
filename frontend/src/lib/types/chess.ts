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
	lastMove?: LastMove;
}

export interface LastMove {
	from: string;
	to: string;
	piece: string;
	color: string;
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

export interface PromotionChoice {
	from: string;
	to: string;
	piece: 'queen' | 'rook' | 'bishop' | 'knight';
}

export interface PendingPromotion {
	from: string;
	to: string;
	isActive: boolean;
}

export interface GameState {
	user: User | null;
	userProfile: UserProfile | null;
	currentGame: Game | null;
	loading: boolean;
	error: string | null;
	gameStartTime: number | null;
	elapsedTime: number;
	gameTimer: number | null;
	selectedSquare: string | null;
	possibleMoves: string[];
	pendingPromotion: PendingPromotion | null;
	lastMove: { from: string; to: string } | null; // Change type to match what we use
}

export interface BoardRef {
	setDests(dests: Record<string, string[]>): void;
	clearDests(): void;
	resetToFen(fen: string): void;
	setLastMove(from: string, to: string): void;
}

export type SelectEvent = CustomEvent<{ square: string }>;
export type MoveEvent = CustomEvent<{ from: string; to: string }>;
export type PromoteEvent = CustomEvent<{ piece: 'queen' | 'rook' | 'bishop' | 'knight' } | string>;

export interface ChessgroundInstance {
	set(cfg: Record<string, unknown>): void;
	destroy?: () => void;
}

export type CGEvents = {
	move: { from: string; to: string };
	select: { square: string };
	ready: {
		setDests: (dests: Record<string, string[]>) => void;
		clearDests: () => void;
		setLastMove: (from?: string, to?: string) => void;
	};
};

export interface VerboseMove {
	from: string;
	to: string;
	san?: string;
	flags?: string;
	piece?: string;
	promotion?: string;
}
