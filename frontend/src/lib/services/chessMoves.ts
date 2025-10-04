import { Chess } from 'chess.js';
import type { ChessSquare } from '$lib/types/chess';

/**
 * Use chess.js to compute legal moves for a square given the FEN.
 * Returns an array of destination square ids like ['e4','d5',...]
 */
export function getPossibleMoves(_: ChessSquare[][], square: ChessSquare, fen?: string): string[] {
    if (!square || !square.piece) return [];
    try {
        const chess = new Chess(fen || undefined);
        // chess.js move generator: filter by from squares
        const moves = chess.moves({ square: `${square.file}${square.rank}`, verbose: true });
        // Map verbose moves to destination squares (include promotions as bare to-square)
        return moves.map((m: any) => m.to);
    } catch (err) {
        console.error('chess.js move generation failed', err);
        return [];
    }
}
