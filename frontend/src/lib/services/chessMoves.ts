import { Chess } from 'chess.js';
import type { ChessSquare, VerboseMove } from '$lib/types/chess';

/**
 * Use chess.js to compute legal moves for a square given the FEN.
 * Returns an array of destination square ids like ['e4','d5',...]
 */
export function getPossibleMoves(_: ChessSquare[][], square: ChessSquare, fen?: string): string[] {
	if (!square || !square.piece) return [];
	try {
		// validate FEN format (must have 6 fields). If malformed, warn once and fall back to default position.
		let safeFen = fen;
		if (fen) {
			const parts = fen.trim().split(/\s+/);
			if (parts.length !== 6) {
				console.warn('[moves] malformed FEN — falling back to default position');
				safeFen = undefined;
			}
		}

		const chess = new Chess(safeFen);
		const moves = chess.moves({
			square: `${square.file}${square.rank}`,
			verbose: true
		}) as VerboseMove[];

		return moves.map((m) => m.to);
	} catch (err) {
		console.error('chess.js move generation failed', err);
		return [];
	}
}
