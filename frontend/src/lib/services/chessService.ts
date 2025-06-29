import type { ChessSquare, ChessPiece } from '$lib/types/chess';

export class ChessService {
    /**
     * Parse FEN to get board representation
     */
    static parseFEN(fen: string): ChessSquare[][] {
        const [position] = fen.split(' ');
        const ranks = position.split('/');
        const board: ChessSquare[][] = [];

        for (let rankIndex = 0; rankIndex < 8; rankIndex++) {
            const rank: ChessSquare[] = [];
            const rankString = ranks[rankIndex];
            let fileIndex = 0;

            for (const char of rankString) {
                if (char >= '1' && char <= '8') {
                    // Empty squares
                    const emptySquares = parseInt(char);
                    for (let i = 0; i < emptySquares; i++) {
                        rank.push({
                            file: String.fromCharCode(97 + fileIndex), // a-h
                            rank: 8 - rankIndex, // 8-1
                            piece: undefined
                        });
                        fileIndex++;
                    }
                } else {
                    // Piece
                    const piece = this.charToPiece(char);
                    rank.push({
                        file: String.fromCharCode(97 + fileIndex),
                        rank: 8 - rankIndex,
                        piece
                    });
                    fileIndex++;
                }
            }
            board.push(rank);
        }

        return board;
    }

    /**
     * Convert FEN character to piece
     */
    private static charToPiece(char: string): ChessPiece {
        const isWhite = char === char.toUpperCase();
        const pieceType = char.toLowerCase();

        const typeMap: Record<string, ChessPiece['type']> = {
            p: 'pawn',
            r: 'rook',
            n: 'knight',
            b: 'bishop',
            q: 'queen',
            k: 'king'
        };

        return {
            type: typeMap[pieceType],
            color: isWhite ? 'white' : 'black'
        };
    }

    /**
     * Convert square notation to move string (e.g., e2->e4 = "e2e4")
     */
    static squaresToMove(from: string, to: string): string {
        return `${from}${to}`;
    }

    /**
     * Parse moves JSON string to array
     */
    static parseMoves(movesJson: string): string[] {
        try {
            return JSON.parse(movesJson);
        } catch {
            return [];
        }
    }
}