import { writable } from 'svelte/store';
import type { User, Game, GameMoveResult } from '$lib/types/chess';

interface GameState {
    user: User | null;
    currentGame: Game | null;
    loading: boolean;
    error: string | null;
    selectedSquare: string | null;
    possibleMoves: string[];
}

const initialState: GameState = {
    user: null,
    currentGame: null,
    loading: false,
    error: null,
    selectedSquare: null,
    possibleMoves: []
};

export const gameStore = writable<GameState>(initialState);

export const gameActions = {
    setUser: (user: User) => {
        gameStore.update((state) => ({ ...state, user }));
    },

    setCurrentGame: (game: Game) => {
        gameStore.update((state) => ({ ...state, currentGame: game }));
    },

    updateGameAfterMove: (result: GameMoveResult) => {
        gameStore.update((state) => ({
            ...state,
            currentGame: result.game,
            selectedSquare: null,
            possibleMoves: []
        }));
    },

    selectSquare: (square: string | null) => {
        gameStore.update((state) => ({ ...state, selectedSquare: square }));
    },

    setPossibleMoves: (moves: string[]) => {
        gameStore.update((state) => ({ ...state, possibleMoves: moves }));
    },

    setLoading: (loading: boolean) => {
        gameStore.update((state) => ({ ...state, loading }));
    },

    setError: (error: string | null) => {
        gameStore.update((state) => ({ ...state, error }));
    },

    clearGame: () => {
        gameStore.update((state) => ({
            ...state,
            currentGame: null,
            selectedSquare: null,
            possibleMoves: []
        }));
    }
};