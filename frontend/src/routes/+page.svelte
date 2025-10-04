<script lang="ts">
    import { onDestroy } from 'svelte';
    import { gameStore, gameActions } from '$lib/stores/gameStore';
    import { ChessService } from '$lib/services/chessService';
    import ChessGround from '$lib/components/ChessBoard/ChessGround.svelte';
    import PromotionDialog from '$lib/components/ChessBoard/PromotionDialog.svelte';
    import { getPossibleMoves } from '$lib/services/chessMoves';
    import type { ChessSquare, ChessPiece } from '$lib/types/chess';
    const START_FEN = 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1';

    let username = '';
    let difficulty = 5;
    let gameStarted = false;
    let showStats = false;
    let showLeaderboard = false;
    let leaderboard = [];

    let chessgroundRef: any = null;
    let lastSyncedLastMove: { from: string; to: string } | null = null;
    // Local cache for last selection dests to validate moves synchronously
    let lastSelectedFrom: string | null = null;
    let lastSelectedDests: string[] = [];

    let pendingPromotion: { from: string; to: string } | null = null;

	/**
	 * Helper: detect a pawn promotion client-side (simple check)
	 * @param from
	 * @param to
	 */
    function isPawnPromotion(from: string, to: string): boolean {
        if (!from || !to) return false;
        const fromRank = parseInt(from[1]);
        const toRank = parseInt(to[1]);
        return (fromRank === 7 && toRank === 8) || (fromRank === 2 && toRank === 1);
    }

    /**
	* Return which side is to move from FEN
	* @param fen
	*/
    function turnFromFen(fen: string): 'white' | 'black' {
        if (!fen) return 'white';
        const parts = fen.split(' ');
        return (parts[1] === 'w') ? 'white' : 'black';
    }

	/**
	 * Find a square by its ID (e.g., "e4") in a 2D board array
	 * @param board
	 * @param id
	 */
    function findSquareById(board: ChessSquare[][], id: string): ChessSquare | null {
        for (const row of board) {
            for (const sq of row) {
                if (`${sq.file}${sq.rank}` === id) return sq;
            }
        }
        return null;
    }

	/**
	 * Get possible moves for a square given current FEN
	 * @param squareId
	 */
    async function getPossibleMovesForSquare(squareId: string): Promise<string[]> {
        try {
            const fen = $gameStore.currentGame?.fen ?? START_FEN;
            const board = ChessService.parseFEN(fen);
            const square = findSquareById(board, squareId);
            if (!square) return [];
            const toMove = turnFromFen(fen);
            if (!square.piece || square.piece.color !== toMove) return [];
            const moves = getPossibleMoves(board, square, fen);
            return moves || [];
        } catch (e) {
            console.error('Failed to compute possible moves for', squareId, e);
            return [];
        }
    }

    /**
	 * Handler for select events from chessground
	 * @param e
	 */
    async function onSelectFromBoard(e) {
        const from = e.detail.square;
        console.log('[BOARD] select', from);
        gameActions.selectSquare(from);
        const fen = $gameStore.currentGame?.fen ?? START_FEN;
        const toMove = turnFromFen(fen);
        const board = ChessService.parseFEN(fen);
        const sq = findSquareById(board, from);
        if (!sq || !sq.piece) {
            console.log('[PAGE] no square or piece at', from);
            chessgroundRef?.clearDests();
            gameActions.setPossibleMoves([]);
            return;
        }
        if (sq.piece.color !== toMove) {
            console.log('[PAGE] not side to move for', from, 'pieceColor', sq.piece.color, 'toMove', toMove);
            chessgroundRef?.clearDests();
            gameActions.setPossibleMoves([]);
            return;
        }
        const dests = await getPossibleMovesForSquare(from);
        console.log('[BOARD] possible dests for', from, dests);
        gameActions.setPossibleMoves(dests);
        lastSelectedFrom = from;
        lastSelectedDests = dests;
        chessgroundRef?.setDests({ [from]: dests });
    }

    /**
	 * Handler for move events from chessground
	 * @param e
	 */
    function onMoveFromBoard(e) {
        const { from, to } = e.detail;
        console.log('[BOARD] move event', from, '→', to);
        if (lastSelectedFrom !== from || !lastSelectedDests.includes(to)) {
            console.warn('[BOARD] synchronous guard blocked move', from, '→', to, 'cachedFrom:', lastSelectedFrom, 'dests:', lastSelectedDests);
            chessgroundRef?.resetToFen($gameStore.currentGame?.fen ?? START_FEN);
            // clear cache
            lastSelectedFrom = null;
            lastSelectedDests = [];
            return;
        }

        // Defensive: ensure this move was allowed by our move generator (async check fallback)
        (async () => {
            const fen = $gameStore.currentGame?.fen ?? START_FEN;
            const toMove = turnFromFen(fen);
            // Only allow moves if it's player's turn (white)
            if (toMove !== 'white') {
                console.warn('[BOARD] move blocked (not player turn)', toMove);
                chessgroundRef?.resetToFen($gameStore.currentGame?.fen ?? START_FEN);
                lastSelectedFrom = null;
                lastSelectedDests = [];
                return;
            }
            const allowed = await getPossibleMovesForSquare(from);
            if (!allowed.includes(to)) {
                console.warn('[BOARD] move blocked (not in allowed moves)', from, '→', to, 'allowed:', allowed);
                // reset board to server FEN to cancel illegal movement
                chessgroundRef?.resetToFen($gameStore.currentGame?.fen ?? START_FEN);
                lastSelectedFrom = null;
                lastSelectedDests = [];
                return;
            }
            const boardForPromotion = ChessService.parseFEN(fen);
            const fromSqForPromotion = findSquareById(boardForPromotion, from);
            if (fromSqForPromotion?.piece?.type === 'pawn' && isPawnPromotion(from, to)) {
                pendingPromotion = { from, to };
                chessgroundRef?.clearDests();
                return;
            }
            await makeMove(from, to);
        })();
    }

    /**
	 * Promotion dialog handler
	 * @param e
	 */
    async function onPromote(e) {
        const piece = e.detail.piece || e.detail; // depending on dialog payload
        if (!pendingPromotion) return;
        await makeMove(pendingPromotion.from, pendingPromotion.to, piece);
        pendingPromotion = null;
    }


    /**
        </div>
</main>s a new user account and loads their profile
     */
    async function createUser() {
        if (!username.trim()) {
            gameActions.setError('Please enter a username');
            return;
        }

        gameActions.setLoading(true);
        gameActions.setError(null);

        try {
            const user = await ChessService.createUser(username.trim());
            gameActions.setUser(user);
            await loadUserProfile();
            await loadLeaderboard();
            console.log('👤 User created:', user);
        } catch (error) {
            gameActions.setError(`Failed to create user: ${error.message}`);
            console.error('❌ User creation error:', error);
        } finally {
            gameActions.setLoading(false);
        }
    }

    /**
     * Loads complete user profile with stats and records
     */
    async function loadUserProfile() {
        if (!$gameStore.user) return;

        try {
            const profile = await ChessService.getUserProfile($gameStore.user.id);
            gameActions.setUserProfile(profile);
            console.log('📊 Profile loaded:', profile);
        } catch (error) {
            console.error('❌ Profile loading error:', error);
        }
    }

    /**
     * Loads leaderboard data
     */
    async function loadLeaderboard() {
        try {
            leaderboard = await ChessService.getLeaderboard(10);
            console.log('🏆 Leaderboard loaded:', leaderboard);
        } catch (error) {
            console.error('❌ Leaderboard loading error:', error);
        }
    }

    /**
     * Starts a new chess game against Stockfish
     */
    async function startNewGame() {
        if (!$gameStore.user) return;

        gameActions.setLoading(true);
        gameActions.setError(null);

        try {
            const game = await ChessService.createGame($gameStore.user.id, difficulty);
            gameActions.setCurrentGame(game);
            gameActions.startTimer();
            gameStarted = true;
            console.log('🎮 Game started:', game);
        } catch (error) {
            gameActions.setError(`Failed to start game: ${error.message}`);
            console.error('❌ Game creation error:', error);
        } finally {
            gameActions.setLoading(false);
        }
    }

    /**
     * Makes a chess move with optional promotion support
     * @param from - Source square (e.g., "e2")
     * @param to - Target square (e.g., "e4") 
     * @param promotion - Optional promotion piece ("queen", "rook", "bishop", "knight")
     */
    async function makeMove(from, to, promotion = null) {
        if (!$gameStore.currentGame) return;

        // Format move with promotion
        let playerMove = `${from}${to}`;
        if (promotion) {
            const promotionLetters = {
                'queen': 'q',
                'rook': 'r', 
                'bishop': 'b',
                'knight': 'n'
            };
            playerMove += promotionLetters[promotion];
            console.log('🎯 Pawn promotion:', from, '→', to, 'promoted to', promotion);
        }

        gameActions.setLoading(true);
        gameActions.setError(null);

        try {
            const result = await ChessService.makeMove($gameStore.currentGame.id, playerMove);
            console.log('[MAKE MOVE] server result:', result);
            gameActions.updateGameAfterMove(result);

            console.log('♟️ Move:', playerMove, '→ Stockfish:', result.stockfishMove);

            if (result.gameOver) {
                const outcome = result.winner === 'white' ? 'You won! 🏆' : 
                               result.winner === 'black' ? 'You lost! 😔' : 
                               'Draw! 🤝';
                
                const time = gameActions.formatTime(result.totalTimeSeconds || $gameStore.elapsedTime);
                alert(`${outcome}\n⏱️ Time: ${time}\n♟️ Moves: ${result.game.movesCount}`);
                
                // Reload stats after game ends
                await loadUserProfile();
                await loadLeaderboard();
            }
        } catch (error) {
            gameActions.setError(`Invalid move: ${error.message}`);
            console.error('❌ Move error:', error);
        } finally {
            gameActions.setLoading(false);
        }
    }

    /**
     * Resets game state and returns to menu
     */
    function resetGame() {
        gameActions.clearGame();
        gameStarted = false;
    }

    /**
     * Calculates win rate percentage
     */
    function getWinRate(gamesWon, totalGames) {
        if (totalGames === 0) return 0;
        return Math.round((gamesWon / totalGames) * 100);
    }

    /**
     * Permet d'abandonner la partie (résignation)
     */
    async function resignGame() {
        if (!$gameStore.currentGame || $gameStore.currentGame.status === 'finished') return;
        const updatedGame = {
            ...$gameStore.currentGame,
            status: 'finished',
            result: 'black', // Stockfish gagne
            end_time: new Date().toISOString(),
        };
        gameActions.setCurrentGame(updatedGame);
        gameActions.stopTimer();
        gameStarted = false;
        alert('You resigned! Stockfish wins.');
        await loadUserProfile();
        await loadLeaderboard();
    }

    // Affichage automatique de la fin de partie (mat, pat, abandon)
    let gameOverAlerted = false;
    $: if ($gameStore.currentGame && $gameStore.currentGame.status === 'finished' && !gameOverAlerted) {
        gameOverAlerted = true;
        let outcome = 'Game over!';
        if ($gameStore.currentGame.result === 'white') outcome = 'You won! 🏆';
        else if ($gameStore.currentGame.result === 'black') outcome = 'You lost! 😔';
        else if ($gameStore.currentGame.result === 'draw') outcome = 'Draw! 🤝';
        const time = gameActions.formatTime($gameStore.elapsedTime);
        alert(`${outcome}\n⏱️ Time: ${time}\n♟️ Moves: ${$gameStore.currentGame.movesCount}`);
        // Reload stats after game ends
        loadUserProfile();
        loadLeaderboard();
    }
    $: if ($gameStore.currentGame && $gameStore.currentGame.status === 'active') {
        gameOverAlerted = false;
    }

    // Debug: log current game state whenever it changes (helps verify we received the FEN)
    $: if ($gameStore.currentGame) {
        console.log('[DEBUG] $gameStore.currentGame changed:', $gameStore.currentGame);
        // also log lastMove and possibleMoves
        console.log('[DEBUG] lastMove', $gameStore.lastMove, 'possibleMoves', $gameStore.possibleMoves);
    }

    // Sync lastMove from the store to the chessground instance so highlight shows the
    // most recent move regardless of color. The backend/gameStore provides the lastMove
    // representing the previous played move (should be the last move made, not two moves ago).
    $: if (chessgroundRef && $gameStore.lastMove) {
        const lm = $gameStore.lastMove;
        // Only sync if changed
        if (!lastSyncedLastMove || lastSyncedLastMove.from !== lm.from || lastSyncedLastMove.to !== lm.to) {
            console.log('[BOARD] syncing lastMove to chessground', lm);
            try {
                chessgroundRef.setLastMove(lm.from, lm.to);
                lastSyncedLastMove = { from: lm.from, to: lm.to };
            } catch (err) {
                console.warn('Failed to set lastMove on chessground', err);
            }
        }
    }

    // Cleanup timer on component destroy
    onDestroy(() => {
        gameActions.stopTimer();
    });
</script>

<!-- Fullscreen welcome with chess background and card -->
<div class="min-h-screen bg-background flex items-center justify-center p-4 relative overflow-hidden">
  <!-- Background chess pattern -->
  <div class="absolute inset-0 opacity-5 pointer-events-none">
    <div class="grid grid-cols-8 h-full">
      {#each Array(64) as _, i}
        <div class={Math.floor(i / 8) % 2 === i % 2 ? 'bg-primary' : 'bg-background'}></div>
      {/each}
    </div>
  </div>

  <div class="relative z-10 w-full max-w-md">
    <!-- Chess pieces decoration -->
    <div class="absolute -top-20 left-1/2 -translate-x-1/2 text-6xl opacity-20 select-none">♔</div>
    <div class="absolute -top-16 -left-8 text-4xl opacity-15 select-none">♛</div>
    <div class="absolute -top-16 -right-8 text-4xl opacity-15 select-none">♜</div>

    <!-- Card -->
    <div class="p-8 rounded-xl bg-card-80 backdrop-blur-sm border border-border-50 shadow-2xl">
      <div class="text-center space-y-6">
        <!-- Logo/Title -->
        <div class="space-y-2">
          <div class="text-4xl font-bold text-primary tracking-tight">ChessClub</div>
          <div class="flex justify-center space-x-2 text-2xl opacity-60">
            <span>♔</span>
            <span>♕</span>
            <span>♖</span>
            <span>♗</span>
            <span>♘</span>
            <span>♙</span>
          </div>
        </div>

        <!-- Main heading -->
        <div class="space-y-3">
          <h1 class="text-2xl font-semibold text-foreground">Create your profile</h1>
          <p class="text-muted-foreground leading-relaxed">
            Enter a username to start playing and tracking your progress!
          </p>
        </div>

    <!-- Game in Progress -->
    {:else}
        <div class="game-area">
            <!-- Game Header -->
            <div class="game-header-info">
                <div class="game-details">
                    <h2>♟️ vs Stockfish (Level {$gameStore.currentGame?.difficulty})</h2>
                    <p>Status: <strong>{$gameStore.currentGame?.status}</strong></p>
                    <p>Moves: <strong>{$gameStore.currentGame?.movesCount}</strong></p>
                </div>
                
                <div class="game-timer-large">
                    <div class="timer-display">
                        ⏱️ {gameActions.formatTime($gameStore.elapsedTime)}
                    </div>
                    <div class="timer-label">Game Time</div>
                </div>
            </div>

            <!-- Chess Board -->
            <div class="board-container">
                <!--  <ChessBoard lastMove={$gameStore.lastMove} onMove={makeMove} allowMoves={$gameStore.currentGame?.status === 'active'} /> -->
                                        <ChessGround
                                            bind:this={chessgroundRef}
                                            fen={$gameStore.currentGame?.fen ?? START_FEN}
                                            orientation="white"
                                            viewOnly={$gameStore.currentGame?.status !== 'active'}
                                            on:move={onMoveFromBoard}
                                            on:select={onSelectFromBoard}
                                        />

                                        <!-- Promotion Dialog (géré localement ici) -->
                                        <PromotionDialog
                                            visible={!!pendingPromotion}
                                            from={pendingPromotion?.from}
                                            to={pendingPromotion?.to}
                                            on:promote={onPromote}
                                            on:cancel={() => { pendingPromotion = null; chessgroundRef?.clearDests(); }}
                                        />
            </div>

            <!-- Debug FEN removed for cleaner UI -->

            <!-- Game Controls -->
            <div class="game-controls">
                <button on:click={resetGame} class="btn btn-secondary">
                    🏠 Back to Menu
                </button>
                <button on:click={resignGame} class="btn btn-secondary" disabled={$gameStore.currentGame?.status !== 'active'}>
                    🚩 Resign
                </button>
            </div>
        </div>
    {/if}

    <!-- Statistics Panel -->
    {#if showStats && $gameStore.userProfile}
        <div class="stats-panel">
            <h2>📊 Your Statistics</h2>
            
            <!-- Personal Records -->
            {#if $gameStore.userProfile.records.length > 0}
                <div class="records-section">
                    <h3>🏆 Personal Records</h3>
                    <div class="records-grid">
                        {#each $gameStore.userProfile.records as record}
                            <div class="record-card">
                                <div class="record-level">Level {record.difficulty}</div>
                                <div class="record-time">⏱️ {gameActions.formatTime(record.bestTimeSeconds)}</div>
                                <div class="record-moves">♟️ {record.movesCount} moves</div>
                            </div>
                        {/each}
                    </div>
                </div>
            {/if}
          </div>

          {#if username}
            <div class="text-sm text-muted-foreground">
              Welcome, <span class="font-medium text-foreground">{username}</span>!
            </div>
          {/if}

          <button
            on:click={createUser}
            class="w-full h-12 rounded-full bg-primary text-white font-semibold disabled:opacity-60"
            disabled={$gameStore.loading || !username.trim()}
          >
            🚀 Create my profile
          </button>
        </div>

        <!-- Decorative elements -->
        <div class="pt-4 border-t border-border-30">
          <div class="flex justify-center space-x-4 text-xs text-muted-foreground">
            <span>Strategy</span>
            <span>•</span>
            <span>Tactics</span>
            <span>•</span>
            <span>Mastery</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Bottom decoration -->
    <div class="absolute -bottom-12 left-1/2 -translate-x-1/2 flex space-x-4 text-2xl opacity-20 select-none">
      <span>♞</span>
      <span>♝</span>
      <span>♟</span>
    </div>
  </div>
</div>

<style>
  .opacity-15 { opacity: 0.15; }
</style>