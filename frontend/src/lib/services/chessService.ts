import { executeGraphQL } from '$lib/graphql/client';
import { 
    CREATE_USER, 
    CREATE_GAME, 
    MAKE_MOVE, 
    GET_USER_PROFILE, 
    GET_LEADERBOARD 
} from '$lib/graphql/queries';
import type { User, Game, UserProfile, GameMoveResult } from '$lib/types/chess';

/**
 * Chess service for handling all game-related API calls
 */
export class ChessService {
    /**
     * Creates a new user account
     * @param username - Desired username (must be unique)
     * @returns Promise with created user data
     * @throws Error if username is taken or invalid
     */
    static async createUser(username: string): Promise<User> {
        const result = await executeGraphQL(CREATE_USER, { username });
        return result.createUser;
    }

    /**
     * Starts a new chess game against Stockfish
     * @param userId - User ID who will play the game
     * @param difficulty - Stockfish difficulty level (1-20)
     * @returns Promise with new game data including initial position
     * @throws Error if user doesn't exist or invalid difficulty
     */
    static async createGame(userId: string, difficulty: number): Promise<Game> {
        const result = await executeGraphQL(CREATE_GAME, {
            input: { userId, difficulty }
        });
        return result.createGame;
    }

    /**
     * Makes a move in the current game
     * @param gameId - ID of the active game
     * @param playerMove - Move in algebraic notation (e.g., "e2e4", "Nf3")
     * @returns Promise with move result including Stockfish response
     * @throws Error if move is invalid or game is finished
     */
    static async makeMove(gameId: string, playerMove: string): Promise<GameMoveResult> {
        const result = await executeGraphQL(MAKE_MOVE, {
            input: { gameId, playerMove }
        });
        return result.makeMove;
    }

    /**
     * Retrieves complete user profile with statistics and records
     * @param userId - User ID to fetch profile for
     * @returns Promise with user profile including stats and personal records
     * @throws Error if user doesn't exist
     */
    static async getUserProfile(userId: string): Promise<UserProfile> {
        const result = await executeGraphQL(GET_USER_PROFILE, { userId });
        return result.getUserProfile;
    }

    /**
     * Gets leaderboard of top players ranked by ELO
     * @param limit - Maximum number of players to return (default: 10)
     * @returns Promise with array of top players
     */
    static async getLeaderboard(limit: number = 10): Promise<User[]> {
        const result = await executeGraphQL(GET_LEADERBOARD, { limit });
        return result.getLeaderboard;
    }

    /**
     * Parses FEN notation into a board representation
     * @param fen - FEN string representing the board position
     * @returns 8x8 array of ChessSquare objects
     */
    static parseFEN(fen: string): ChessSquare[][] {
        const [position] = fen.split(' ');
        const ranks = position.split('/');
        const board: ChessSquare[][] = [];

        ranks.forEach((rank, rankIndex) => {
            const row: ChessSquare[] = [];
            let fileIndex = 0;

            for (const char of rank) {
                if (char >= '1' && char <= '8') {
                    // Empty squares
                    const emptyCount = parseInt(char);
                    for (let i = 0; i < emptyCount; i++) {
                        row.push({
                            file: String.fromCharCode(97 + fileIndex),
                            rank: 8 - rankIndex,
                            piece: null
                        });
                        fileIndex++;
                    }
                } else {
                    // Piece
                    const isWhite = char === char.toUpperCase();
                    const pieceType = char.toLowerCase();
                    
                    let type: ChessPiece['type'];
                    switch (pieceType) {
                        case 'k': type = 'king'; break;
                        case 'q': type = 'queen'; break;
                        case 'r': type = 'rook'; break;
                        case 'b': type = 'bishop'; break;
                        case 'n': type = 'knight'; break;
                        case 'p': type = 'pawn'; break;
                        default: type = 'pawn'; break;
                    }

                    row.push({
                        file: String.fromCharCode(97 + fileIndex),
                        rank: 8 - rankIndex,
                        piece: {
                            type,
                            color: isWhite ? 'white' : 'black'
                        }
                    });
                    fileIndex++;
                }
            }
            board.push(row);
        });

        return board;
    }
}