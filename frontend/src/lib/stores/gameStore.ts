import { writable, get } from 'svelte/store';
import type { Game, PendingPromotion, User, UserProfile, LastMove } from '$lib/types/chess';
import { ChessService } from '$lib/services/chessService';

/**
 * Game state interface managing user data, current game, and timer functionality
 */
interface GameState {
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

const initialState: GameState = {
    user: null,
    userProfile: null,
    currentGame: null,
    loading: false,
    error: null,
    gameStartTime: null,
    elapsedTime: 0,
    gameTimer: null,
    selectedSquare: null,
    possibleMoves: [],
    pendingPromotion: null,
    lastMove: null // Ajout pour animation du dernier coup
};


export const gameStore = writable<GameState>(initialState);

/**
 * Game store actions for managing chess game state and timer functionality
 */
export const gameActions = {

        /**
     * Sets the selected square for move highlighting
     * @param square - Square notation (e.g., "e2") or null to clear
     */
    selectSquare: (square: string | null) => {
        gameStore.update(state => ({ ...state, selectedSquare: square }));
    },
    
    /**
     * Sets possible moves for the selected piece
     * @param moves - Array of possible move squares
     */
    setPossibleMoves: (moves: string[]) => {
        gameStore.update(state => ({ ...state, possibleMoves: moves }));
    },

    /**
     * Sets a pending pawn promotion
     * @param from - Source square
     * @param to - Target square (8th rank)
     */
    setPendingPromotion: (from: string, to: string) => {
        gameStore.update(state => ({ 
            ...state, 
            pendingPromotion: { from, to, isActive: true } 
        }));
    },

    /**
     * Clears pending promotion
     */
    clearPendingPromotion: () => {
        gameStore.update(state => ({ 
            ...state, 
            pendingPromotion: null 
        }));
    },
    

    /**
     * Sets the current user data
     * @param user - User object containing profile information
     */
    setUser: (user: User) => {
        gameStore.update(state => ({ ...state, user }));
    },
    
    /**
     * Updates user profile with stats and records
     * @param userProfile - Complete user profile with statistics
     */
    setUserProfile: (userProfile: UserProfile) => {
        gameStore.update(state => ({ ...state, userProfile }));
    },
    
    /**
     * Sets the current active game
     * @param game - Game object with current position and status
     */
    setCurrentGame: (game: Game) => {
        gameStore.update(state => ({ ...state, currentGame: game }));
    },
    
    /**
     * Updates loading state for UI feedback
     * @param loading - Boolean indicating if an operation is in progress
     */
    setLoading: (loading: boolean) => {
        gameStore.update(state => ({ ...state, loading }));
    },
    
    /**
     * Sets error message for user feedback
     * @param error - Error message string or null to clear
     */
    setError: (error: string | null) => {
        gameStore.update(state => ({ ...state, error }));
    },
    
    /**
     * Starts the game timer and updates elapsed time every second
     * Automatically clears any existing timer before starting new one
     */
    startTimer: () => {
        gameStore.update(state => {
            // Clear existing timer if any
            if (state.gameTimer) clearInterval(state.gameTimer);
            
            const startTime = Date.now();
            const timer = setInterval(() => {
                gameStore.update(s => ({
                    ...s,
                    elapsedTime: Math.floor((Date.now() - startTime) / 1000)
                }));
            }, 1000);
            
            return {
                ...state,
                gameStartTime: startTime,
                elapsedTime: 0,
                gameTimer: timer
            };
        });
    },
    
    /**
     * Stops the game timer without clearing elapsed time
     * Used when game is paused or finished
     */
    stopTimer: () => {
        gameStore.update(state => {
            if (state.gameTimer) {
                clearInterval(state.gameTimer);
            }
            return {
                ...state,
                gameTimer: null
            };
        });
    },
    
    /**
     * Updates game state after a move is made
     * Automatically stops timer if game is over
     * @param result - Move result containing updated game state and outcome
     */
    updateGameAfterMove: (result: any) => {
        // Log the FEN after every move
        if (result?.game?.fen) {
            console.log('📥 New FEN after move:', result.game.fen);
        }
        // Détecter le dernier coup joué (si fourni par le backend, sinon à calculer)
        let lastMove: LastMove | null = null;
        if (result?.lastMove) {
            // Backend provided explicit lastMove (preferred)
            lastMove = {
                from: result.lastMove.from,
                to: result.lastMove.to,
                piece: result.lastMove.piece,
                color: result.lastMove.color
            };
        } else if (result?.move) {
            // Legacy/alternate payload
            lastMove = result.move as LastMove;
        } else if (result?.game?.lastMove) {
            lastMove = result.game.lastMove as LastMove;
        }

        // If we still don't have a reliable lastMove, try to compute it by diffing
        // the previous FEN (state.currentGame?.fen) and the new FEN (result.game.fen).
        if (!lastMove) {
            try {
                const prevState = get(gameStore);
                const prevFen = prevState.currentGame?.fen;
                const newFen = result?.game?.fen;
                if (prevFen && newFen && prevFen !== newFen) {
                    const prevBoard = ChessService.parseFEN(prevFen);
                    const newBoard = ChessService.parseFEN(newFen);
                    let fromSq: string | null = null;
                    let toSq: string | null = null;
                    for (const row of prevBoard) {
                        for (const sq of row) {
                            const id = `${sq.file}${sq.rank}`;
                            const prevPiece = sq.piece;
                            const newPiece = (() => {
                                const nbRow = newBoard.find(r => r.some(s => `${s.file}${s.rank}` === id));
                                if (!nbRow) return null;
                                return nbRow.find(s => `${s.file}${s.rank}` === id)?.piece ?? null;
                            })();
                            if (prevPiece && !newPiece) {
                                fromSq = id;
                            }
                            if (!prevPiece && newPiece) {
                                toSq = id;
                            }
                            if (prevPiece && newPiece && (prevPiece.type !== newPiece.type || prevPiece.color !== newPiece.color)) {
                                toSq = id;
                            }
                        }
                    }
                    if (fromSq && toSq) {
                        const piece = newBoard.flat().find(s => `${s.file}${s.rank}` === toSq)?.piece;
                        lastMove = { from: fromSq, to: toSq, piece: piece?.type ?? 'pawn', color: piece?.color ?? 'white' } as LastMove;
                    }
                }
            } catch (err) {
                console.warn('Failed to compute lastMove by FEN diff', err);
            }
        }
        console.log('[GAMESTORE] updateGameAfterMove: result.lastMove =', result?.lastMove, 'result.move =', result?.move, 'result.game.lastMove =', result?.game?.lastMove, 'lastMove used =', lastMove);
        gameStore.update(state => ({
            ...state,
            currentGame: result.game,
            lastMove
        }));
        // Stop timer if game ended
        if (result.gameOver) {
            gameActions.stopTimer();
        }
    },
    
    /**
     * Clears current game and resets timer state
     * Used when starting new game or returning to menu
     */
    clearGame: () => {
        gameStore.update(state => {
            // Cleanup timer
            if (state.gameTimer) clearInterval(state.gameTimer);
            
            return {
                ...state,
                currentGame: null,
                gameStartTime: null,
                elapsedTime: 0,
                gameTimer: null,
                lastMove: null // Reset last move
            };
        });
    },

    /**
     * Formats elapsed time in MM:SS format
     * @param seconds - Number of seconds to format
     * @returns Formatted time string (e.g., "02:45")
     */
    formatTime: (seconds: number): string => {
        const mins = Math.floor(seconds / 60);
        const secs = seconds % 60;
        return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
    }

    
};