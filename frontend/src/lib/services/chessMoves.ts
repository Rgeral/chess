import type { ChessSquare, ChessPiece } from '$lib/types/chess';

const files = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];

export function getPossibleMoves(board: ChessSquare[][], square: ChessSquare, fen?: string): string[] {
    if (!square.piece) return [];
    const { type, color } = square.piece;
    switch (type) {
        case 'bishop':
            return getBishopMoves(board, square, color);
        case 'knight':
            return getKnightMoves(board, square, color);
        case 'pawn':
            return getPawnMoves(board, square, color);
        case 'queen':
            return getQueenMoves(board, square, color);
        case 'king':
            return getKingMoves(board, square, color, fen);
        case 'rook':
            return getRookMoves(board, square, color);
        default:
            return [];
    }
}

function getBishopMoves(board: ChessSquare[][], square: ChessSquare, color: ChessPiece['color']): string[] {
    const moves: string[] = [];
    const directions = [
        [1, 1], [1, -1], [-1, 1], [-1, -1]
    ];
    const fileIdx = files.indexOf(square.file);
    const rankIdx = square.rank;
    for (const [df, dr] of directions) {
        let f = fileIdx + df;
        let r = rankIdx + dr;
        while (f >= 0 && f < 8 && r >= 1 && r <= 8) {
            const dest = getSquare(board, files[f], r);
            if (!dest) break;
            if (!dest.piece) {
                moves.push(`${files[f]}${r}`);
            } else {
                if (dest.piece.color !== color) {
                    moves.push(`${files[f]}${r}`); // capture
                }
                break; // bloqué
            }
            f += df;
            r += dr;
        }
    }
    return moves;
}

function getKnightMoves(board: ChessSquare[][], square: ChessSquare, color: ChessPiece['color']): string[] {
    const moves: string[] = [];
    const fileIdx = files.indexOf(square.file);
    const rankIdx = square.rank;
    const knightMoves = [
        [1, 2], [2, 1], [2, -1], [1, -2],
        [-1, -2], [-2, -1], [-2, 1], [-1, 2]
    ];
    for (const [df, dr] of knightMoves) {
        const f = fileIdx + df;
        const r = rankIdx + dr;
        if (f >= 0 && f < 8 && r >= 1 && r <= 8) {
            const dest = getSquare(board, files[f], r);
            if (!dest) continue;
            if (!dest.piece || dest.piece.color !== color) {
                moves.push(`${files[f]}${r}`);
            }
        }
    }
    return moves;
}

function getPawnMoves(board: ChessSquare[][], square: ChessSquare, color: ChessPiece['color']): string[] {
    const moves: string[] = [];
    const fileIdx = files.indexOf(square.file);
    const rankIdx = square.rank;
    const dir = color === 'white' ? 1 : -1;
    const startRank = color === 'white' ? 2 : 7;
    // Avance d'une case
    const oneForward = getSquare(board, files[fileIdx], rankIdx + dir);
    if (oneForward && !oneForward.piece) {
        moves.push(`${files[fileIdx]}${rankIdx + dir}`);
        // Avance de deux cases depuis la position initiale
        if (rankIdx === startRank) {
            const twoForward = getSquare(board, files[fileIdx], rankIdx + 2 * dir);
            if (twoForward && !twoForward.piece) {
                moves.push(`${files[fileIdx]}${rankIdx + 2 * dir}`);
            }
        }
    }
    // Prises diagonales
    for (const df of [-1, 1]) {
        const f = fileIdx + df;
        if (f >= 0 && f < 8) {
            const diag = getSquare(board, files[f], rankIdx + dir);
            if (diag && diag.piece && diag.piece.color !== color) {
                moves.push(`${files[f]}${rankIdx + dir}`);
            }
        }
    }
    // TODO: prise en passant, promotion
    return moves;
}

function getQueenMoves(board: ChessSquare[][], square: ChessSquare, color: ChessPiece['color']): string[] {
    // La dame combine les mouvements de la tour et du fou
    return [
        ...getBishopMoves(board, square, color),
        ...getRookMoves(board, square, color)
    ];
}

function getRookMoves(board: ChessSquare[][], square: ChessSquare, color: ChessPiece['color']): string[] {
    const moves: string[] = [];
    const directions = [
        [1, 0], [-1, 0], [0, 1], [0, -1]
    ];
    const fileIdx = files.indexOf(square.file);
    const rankIdx = square.rank;
    for (const [df, dr] of directions) {
        let f = fileIdx + df;
        let r = rankIdx + dr;
        while (f >= 0 && f < 8 && r >= 1 && r <= 8) {
            const dest = getSquare(board, files[f], r);
            if (!dest) break;
            if (!dest.piece) {
                moves.push(`${files[f]}${r}`);
            } else {
                if (dest.piece.color !== color) {
                    moves.push(`${files[f]}${r}`); // capture
                }
                break; // bloqué
            }
            f += df;
            r += dr;
        }
    }
    return moves;
}

function getKingMoves(board: ChessSquare[][], square: ChessSquare, color: ChessPiece['color'], fen?: string): string[] {
    const moves: string[] = [];
    const fileIdx = files.indexOf(square.file);
    const rankIdx = square.rank;
    // Déplacements classiques du roi (1 case dans toutes les directions)
    for (let df = -1; df <= 1; df++) {
        for (let dr = -1; dr <= 1; dr++) {
            if (df === 0 && dr === 0) continue;
            const f = fileIdx + df;
            const r = rankIdx + dr;
            if (f >= 0 && f < 8 && r >= 1 && r <= 8) {
                const dest = getSquare(board, files[f], r);
                if (!dest) continue;
                if (!dest.piece || dest.piece.color !== color) {
                    moves.push(`${files[f]}${r}`);
                }
            }
        }
    }
    // Roque (simplifié, à affiner si besoin)
    if (fen) {
        moves.push(...getCastlingMoves(board, square, color, fen));
    }
    return moves;
}

function getCastlingMoves(board: ChessSquare[][], square: ChessSquare, color: ChessPiece['color'], fen: string): string[] {
    // On extrait les droits de roque du FEN
    // FEN format: rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
    // Droits de roque = 3e champ (KQkq)
    const parts = fen.split(' ');
    const castling = parts[2] || '';
    const moves: string[] = [];
    if (color === 'white' && square.file === 'e' && square.rank === 1) {
        // Petit roque blanc
        if (castling.includes('K') &&
            isEmpty(board, 'f', 1) && isEmpty(board, 'g', 1) &&
            !isAttacked(board, 'e', 1, 'black') &&
            !isAttacked(board, 'f', 1, 'black') &&
            !isAttacked(board, 'g', 1, 'black')) {
            moves.push('g1');
        }
        // Grand roque blanc
        if (castling.includes('Q') &&
            isEmpty(board, 'd', 1) && isEmpty(board, 'c', 1) && isEmpty(board, 'b', 1) &&
            !isAttacked(board, 'e', 1, 'black') &&
            !isAttacked(board, 'd', 1, 'black') &&
            !isAttacked(board, 'c', 1, 'black')) {
            moves.push('c1');
        }
    }
    if (color === 'black' && square.file === 'e' && square.rank === 8) {
        // Petit roque noir
        if (castling.includes('k') &&
            isEmpty(board, 'f', 8) && isEmpty(board, 'g', 8) &&
            !isAttacked(board, 'e', 8, 'white') &&
            !isAttacked(board, 'f', 8, 'white') &&
            !isAttacked(board, 'g', 8, 'white')) {
            moves.push('g8');
        }
        // Grand roque noir
        if (castling.includes('q') &&
            isEmpty(board, 'd', 8) && isEmpty(board, 'c', 8) && isEmpty(board, 'b', 8) &&
            !isAttacked(board, 'e', 8, 'white') &&
            !isAttacked(board, 'd', 8, 'white') &&
            !isAttacked(board, 'c', 8, 'white')) {
            moves.push('c8');
        }
    }
    return moves;
}

function isEmpty(board: ChessSquare[][], file: string, rank: number): boolean {
    const sq = getSquare(board, file, rank);
    return !!sq && !sq.piece;
}

// ATTENTION: isAttacked est une version simplifiée qui ne gère pas tous les cas (pas de prise en passant, etc.)
function isAttacked(board: ChessSquare[][], file: string, rank: number, byColor: ChessPiece['color']): boolean {
    // On parcourt toutes les pièces adverses et on regarde si elles peuvent aller sur la case
    for (const row of board) {
        for (const sq of row) {
            if (sq.piece && sq.piece.color === byColor) {
                const moves = getPossibleMoves(board, sq);
                if (moves.includes(`${file}${rank}`)) {
                    return true;
                }
            }
        }
    }
    return false;
}

function getSquare(board: ChessSquare[][], file: string, rank: number): ChessSquare | null {
    for (const row of board) {
        for (const sq of row) {
            if (sq.file === file && sq.rank === rank) return sq;
        }
    }
    return null;
}
