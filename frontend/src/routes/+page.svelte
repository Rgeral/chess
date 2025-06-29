<script lang="ts">
    import { onMount } from 'svelte';
    import { executeGraphQL } from '$lib/graphql/client';
    import { CREATE_USER, CREATE_GAME, MAKE_MOVE } from '$lib/graphql/queries';
    import { gameStore, gameActions } from '$lib/stores/gameStore';
    import ChessBoard from '$lib/components/ChessBoard.svelte';
    import type { User, Game } from '$lib/types/chess';

    let username = '';
    let difficulty = 5;
    let gameStarted = false;

    async function createUser() {
        if (!username.trim()) return;
        
        gameActions.setLoading(true);
        gameActions.setError(null);

        try {
            const result = await executeGraphQL(CREATE_USER, { 
                username: username.trim() 
            });

            const user: User = result.createUser;
            gameActions.setUser(user);
            console.log('✅ User created:', user);
        } catch (error) {
            gameActions.setError(`Failed to create user: ${error}`);
            console.error('❌ User creation error:', error);
        } finally {
            gameActions.setLoading(false);
        }
    }

    async function startNewGame() {
        if (!$gameStore.user) return;

        gameActions.setLoading(true);
        gameActions.setError(null);

        try {
            const result = await executeGraphQL(CREATE_GAME, {
                input: {
                    userId: $gameStore.user.id,
                    difficulty
                }
            });

            const game: Game = result.createGame;
            gameActions.setCurrentGame(game);
            gameStarted = true;
            console.log('🎮 Game started:', game);
        } catch (error) {
            gameActions.setError(`Failed to start game: ${error}`);
            console.error('❌ Game creation error:', error);
        } finally {
            gameActions.setLoading(false);
        }
    }

    async function makeMove(from: string, to: string) {
        if (!$gameStore.currentGame) return;

        const moveNotation = `${from}${to}`;
        gameActions.setLoading(true);
        gameActions.setError(null);

        try {
            const result = await executeGraphQL(MAKE_MOVE, {
                input: {
                    gameId: $gameStore.currentGame.id,
                    playerMove: moveNotation
                }
            });

            const moveResult = result.makeMove;
            gameActions.updateGameAfterMove(moveResult);

            console.log('♟️ Move made:', moveNotation, '-> Stockfish:', moveResult.stockfish_move);

            if (moveResult.gameOver) {
                alert(`Game Over! Winner: ${moveResult.winner || 'Draw'}`);
            }
        } catch (error) {
            gameActions.setError(`Invalid move: ${error}`);
            console.error('❌ Move error:', error);
        } finally {
            gameActions.setLoading(false);
        }
    }

    function resetGame() {
        gameActions.clearGame();
        gameStarted = false;
    }
</script>

<main class="container">
    <h1>🏁 Chess vs Stockfish</h1>

    {#if $gameStore.loading}
        <div class="loading">⏳ Loading...</div>
    {/if}

    {#if $gameStore.error}
        <div class="error">❌ {$gameStore.error}</div>
    {/if}

    {#if !$gameStore.user}
        <!-- User Creation -->
        <div class="user-setup">
            <h2>👤 Create Player</h2>
            <input 
                type="text" 
                bind:value={username} 
                placeholder="Enter your username"
                class="input"
            />
            <button on:click={createUser} disabled={$gameStore.loading} class="btn btn-primary">
                Create Player
            </button>
        </div>
    {:else if !gameStarted}
        <!-- Game Setup -->
        <div class="game-setup">
            <h2>🎮 Welcome, {$gameStore.user.username}!</h2>
            <p>Games: {$gameStore.user.total_games} | Wins: {$gameStore.user.games_won}</p>
            
            <div class="difficulty-selector">
                <label for="difficulty">Stockfish Difficulty (1-20):</label>
                <input 
                    type="range" 
                    id="difficulty" 
                    bind:value={difficulty} 
                    min="1" 
                    max="20" 
                    class="slider"
                />
                <span class="difficulty-value">{difficulty}</span>
            </div>

            <button on:click={startNewGame} disabled={$gameStore.loading} class="btn btn-success">
                🚀 Start New Game
            </button>
        </div>
    {:else}
        <!-- Game in Progress -->
        <div class="game-area">
            <div class="game-info">
                <h2>♟️ Game vs Stockfish (Difficulty {$gameStore.currentGame?.difficulty})</h2>
                <p>Status: <strong>{$gameStore.currentGame?.status}</strong></p>
                {#if $gameStore.selectedSquare}
                    <p>Selected: <strong>{$gameStore.selectedSquare}</strong></p>
                {/if}
            </div>

            <ChessBoard onMove={makeMove} />

            <div class="game-controls">
                <button on:click={resetGame} class="btn btn-secondary">
                    🔄 New Game
                </button>
            </div>
        </div>
    {/if}
</main>

<style>
    .container {
        max-width: 800px;
        margin: 0 auto;
        padding: 20px;
        text-align: center;
    }

    .user-setup, .game-setup, .game-area {
        margin: 20px 0;
        padding: 20px;
        border: 1px solid #ddd;
        border-radius: 8px;
        background: #f9f9f9;
    }

    .input {
        padding: 10px;
        margin: 10px;
        border: 1px solid #ccc;
        border-radius: 4px;
        font-size: 16px;
    }

    .btn {
        padding: 12px 24px;
        margin: 10px;
        border: none;
        border-radius: 4px;
        font-size: 16px;
        cursor: pointer;
        transition: background-color 0.2s;
    }

    .btn-primary { background: #007bff; color: white; }
    .btn-primary:hover { background: #0056b3; }
    .btn-success { background: #28a745; color: white; }
    .btn-success:hover { background: #1e7e34; }
    .btn-secondary { background: #6c757d; color: white; }
    .btn-secondary:hover { background: #545b62; }

    .btn:disabled {
        opacity: 0.6;
        cursor: not-allowed;
    }

    .difficulty-selector {
        margin: 20px 0;
    }

    .slider {
        width: 200px;
        margin: 0 10px;
    }

    .difficulty-value {
        font-weight: bold;
        color: #007bff;
    }

    .loading {
        color: #ffc107;
        font-weight: bold;
        margin: 10px 0;
    }

    .error {
        color: #dc3545;
        font-weight: bold;
        margin: 10px 0;
        padding: 10px;
        background: #f8d7da;
        border: 1px solid #f5c6cb;
        border-radius: 4px;
    }

    .game-info {
        margin-bottom: 20px;
    }

    .game-controls {
        margin-top: 20px;
    }
</style>