<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { gameStore, gameActions } from '$lib/stores/gameStore';
    import { ChessService } from '$lib/services/chessService';
    import ChessGround from '$lib/components/ChessBoard/ChessGround.svelte';
    import PromotionDialog from '$lib/components/ChessBoard/PromotionDialog.svelte';
    import { getPossibleMoves } from '$lib/services/chessMoves';
    import type { ChessSquare, ChessPiece } from '$lib/types/chess';
    const START_FEN = 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1';

    function onSelect(e) { console.log('select', e.detail); }
    function onMove(e) { console.log('move', e.detail); }

    let username = '';
    let difficulty = 5;
    let gameStarted = false;
    let showStats = false;
    let showLeaderboard = false;
    let leaderboard = [];

    // Reference to the ChessGround wrapper instance (bind:this)
    let chessgroundRef: any = null;
    let lastSyncedLastMove: { from: string; to: string } | null = null;
    // Local cache for last selection dests to validate moves synchronously
    let lastSelectedFrom: string | null = null;
    let lastSelectedDests: string[] = [];

    // Pending promotion state local to the page
    let pendingPromotion: { from: string; to: string } | null = null;

    // Helper: detect a pawn promotion client-side (simple check)
    function isPawnPromotion(from: string, to: string): boolean {
        if (!from || !to) return false;
        const fromRank = parseInt(from[1]);
        const toRank = parseInt(to[1]);
        // Quick check: moving to last rank
        return (fromRank === 7 && toRank === 8) || (fromRank === 2 && toRank === 1);
    }

    // Return which side is to move from FEN
    function turnFromFen(fen: string): 'white' | 'black' {
        if (!fen) return 'white';
        const parts = fen.split(' ');
        return (parts[1] === 'w') ? 'white' : 'black';
    }

    function findSquareById(board: ChessSquare[][], id: string): ChessSquare | null {
        for (const row of board) {
            for (const sq of row) {
                if (`${sq.file}${sq.rank}` === id) return sq;
            }
        }
        return null;
    }

    async function getPossibleMovesForSquare(squareId: string): Promise<string[]> {
        try {
            const fen = $gameStore.currentGame?.fen ?? START_FEN;
            const board = ChessService.parseFEN(fen);
            const square = findSquareById(board, squareId);
            if (!square) return [];
            // Ensure it's the correct side to move
            const toMove = turnFromFen(fen);
            if (!square.piece || square.piece.color !== toMove) return [];
            // call local move generator
            const moves = getPossibleMoves(board, square, fen);
            return moves || [];
        } catch (e) {
            console.error('Failed to compute possible moves for', squareId, e);
            return [];
        }
    }

    // Handler for selection on the chessground board -> request and show destinations
    async function onSelectFromBoard(e) {
        const from = e.detail.square;
        console.log('[BOARD] select', from);
        gameActions.selectSquare(from);
        // Only compute/show dests if it's the piece color to move
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
            // Not this side's turn – don't show dests
            chessgroundRef?.clearDests();
            gameActions.setPossibleMoves([]);
            return;
        }
        const dests = await getPossibleMovesForSquare(from);
        console.log('[BOARD] possible dests for', from, dests);
        gameActions.setPossibleMoves(dests);
        // Cache locally to validate user moves synchronously
        lastSelectedFrom = from;
        lastSelectedDests = dests;
        // chessground expects a mapping { from: ['to1','to2'] }
        chessgroundRef?.setDests({ [from]: dests });
    }

    // Handler for move events from chessground
    function onMoveFromBoard(e) {
        const { from, to } = e.detail;
        console.log('[BOARD] move event', from, '→', to);
        // Synchronous guard: if we have a cached selection, check it immediately
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
            // Assume user always plays white for now; if you support playing black, adapt here.
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
            // Continue with promotion handling and makeMove
            if (isPawnPromotion(from, to)) {
                pendingPromotion = { from, to };
                chessgroundRef?.clearDests();
                return;
            }
            await makeMove(from, to);
        })();
    }

    // Promotion dialog handler
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
        // Simule la fin de partie côté backend (on pourrait faire une mutation dédiée, ici on fait simple)
        // On met à jour le store localement
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

<main class="container">
    <!-- Header with user info and timer -->
    <header class="game-header">
        <h1>♟️ Chess Master Pro BETA</h1>
        
        {#if $gameStore.user}
            <div class="user-info">
                <div class="user-details">
                    <strong>{$gameStore.user.username}</strong>
                    {#if $gameStore.userProfile?.user.estimatedElo}
                        <span class="elo-badge">ELO {$gameStore.userProfile.user.estimatedElo}</span>
                    {/if}
                </div>
                
                {#if gameStarted && $gameStore.currentGame}
                    <div class="game-timer">
                        ⏱️ {gameActions.formatTime($gameStore.elapsedTime)}
                    </div>
                {/if}
            </div>
        {/if}
    </header>

    <!-- Loading and Error States -->
    {#if $gameStore.loading}
        <div class="loading">⏳ Loading...</div>
    {/if}

    {#if $gameStore.error}
        <div class="error">❌ {$gameStore.error}</div>
    {/if}

    <!-- User Creation -->
    {#if !$gameStore.user}
        <div class="user-setup">
            <h2>👤 Create Your Profile</h2>
            <p>Enter your username to start playing and tracking your progress!</p>
            
            <div class="input-group">
                <input 
                    type="text" 
                    bind:value={username} 
                    placeholder="Enter your username"
                    class="input"
                    maxlength="20"
                />
                <button 
                    on:click={createUser} 
                    disabled={$gameStore.loading || !username.trim()} 
                    class="btn btn-primary"
                >
                    🚀 Create Profile
                </button>
            </div>
        </div>

    <!-- Main Menu -->
    {:else if !gameStarted}
        <div class="main-menu">
            <!-- Welcome Section -->
            <div class="welcome-section">
                <h2>🎮 Welcome back, {$gameStore.user.username}!</h2>
                
                {#if $gameStore.userProfile}
                    <div class="quick-stats">
                        <div class="stat-item">
                            <span class="stat-value">{$gameStore.userProfile.user.totalGames}</span>
                            <span class="stat-label">Games Played</span>
                        </div>
                        <div class="stat-item">
                            <span class="stat-value">{getWinRate($gameStore.userProfile.user.gamesWon, $gameStore.userProfile.user.totalGames)}%</span>
                            <span class="stat-label">Win Rate</span>
                        </div>
                        <div class="stat-item">
                            <span class="stat-value">{$gameStore.userProfile.user.currentStreak || 0}</span>
                            <span class="stat-label">Current Streak</span>
                        </div>
                        <div class="stat-item">
                            <span class="stat-value">{$gameStore.userProfile.user.estimatedElo || 800}</span>
                            <span class="stat-label">ELO Rating</span>
                        </div>
                    </div>
                {/if}
            </div>

            <!-- Game Setup -->
            <div class="game-setup">
                <h3>🏁 Start New Game</h3>
                
                <div class="difficulty-selector">
                    <label for="difficulty">
                        Stockfish Difficulty: <strong>{difficulty}</strong>
                        <span class="difficulty-desc">
                            {difficulty <= 5 ? 'Beginner' : difficulty <= 10 ? 'Intermediate' : difficulty <= 15 ? 'Advanced' : 'Expert'}
                        </span>
                    </label>
                    <input 
                        type="range" 
                        id="difficulty" 
                        bind:value={difficulty} 
                        min="1" 
                        max="20" 
                        class="slider"
                    />
                    <div class="difficulty-range">
                        <span>Easy (1)</span>
                        <span>Master (20)</span>
                    </div>
                </div>

                <button 
                    on:click={startNewGame} 
                    disabled={$gameStore.loading} 
                    class="btn btn-success btn-large"
                >
                    🚀 Start Game (Level {difficulty})
                </button>
            </div>

            <!-- Menu Actions -->
            <div class="menu-actions">
                <button 
                    on:click={() => showStats = !showStats} 
                    class="btn btn-secondary"
                >
                    📊 {showStats ? 'Hide' : 'View'} Statistics
                </button>
                
                <button 
                    on:click={() => showLeaderboard = !showLeaderboard} 
                    class="btn btn-secondary"
                >
                    🏆 {showLeaderboard ? 'Hide' : 'View'} Leaderboard
                </button>
            </div>
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

            <!-- Debug: show current FEN or warning -->
            <div class="fen-debug">
                {#if $gameStore.currentGame?.fen}
                    <small>FEN: {$gameStore.currentGame.fen}</small>
                {:else}
                    <small style="color: red;">Warning: no FEN available for current game</small>
                {/if}
            </div>

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

            <!-- Level Statistics -->
            {#if $gameStore.userProfile.levelStats.length > 0}
                <div class="level-stats-section">
                    <h3>📈 Performance by Level</h3>
                    <div class="level-stats-grid">
                        {#each $gameStore.userProfile.levelStats as stats}
                            <div class="level-stat-card">
                                <div class="level-header">
                                    <strong>Level {stats.difficulty}</strong>
                                    <span class="win-rate">{getWinRate(stats.gamesWon, stats.gamesPlayed)}%</span>
                                </div>
                                <div class="level-details">
                                    <p>{stats.gamesPlayed} games • {stats.gamesWon} wins</p>
                                    <p>⏱️ Avg: {gameActions.formatTime(stats.averageTimeSeconds)}</p>
                                    <p>♟️ Avg: {stats.averageMoves} moves</p>
                                </div>
                            </div>
                        {/each}
                    </div>
                </div>
            {/if}
        </div>
    {/if}

    <!-- Leaderboard -->
    {#if showLeaderboard && leaderboard.length > 0}
        <div class="leaderboard">
            <h2>🏆 Top Players</h2>
            <div class="leaderboard-list">
                {#each leaderboard as player, index}
                    <div 
                        class="leaderboard-item" 
                        class:current-user={$gameStore.user && player.username === $gameStore.user.username}
                    >
                        <div class="rank">#{index + 1}</div>
                        <div class="player-info">
                            <strong>{player.username}</strong>
                            <span class="player-stats">
                                {player.gamesWon}/{player.totalGames} games
                                {#if player.currentStreak > 0}
                                    • 🔥{player.currentStreak}
                                {/if}
                            </span>
                        </div>
                        <div class="player-elo">{player.estimatedElo} ELO</div>
                    </div>
                {/each}
            </div>
        </div>
    {/if}

    <!-- Promotion Notification -->
    {#if $gameStore.pendingPromotion?.isActive}
        <div class="promotion-notification">
            🎯 Pawn promotion in progress...
        </div>
    {/if}
</main>

<style>
    .container {
        max-width: 1200px;
        margin: 0 auto;
        padding: 20px;
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    }

    /* Header Styles */
    .game-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 25px;
        background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
        color: white;
        border-radius: 20px;
        margin-bottom: 30px;
        box-shadow: 0 10px 30px rgba(0,0,0,0.2);
    }

    .user-info {
        display: flex;
        align-items: center;
        gap: 20px;
    }

    .elo-badge {
        background: rgba(255,255,255,0.2);
        padding: 6px 12px;
        border-radius: 15px;
        font-size: 14px;
        font-weight: bold;
    }

    .game-timer {
        background: #ff6b6b;
        color: white;
        padding: 12px 20px;
        border-radius: 25px;
        font-weight: bold;
        font-size: 18px;
        animation: pulse 2s infinite;
    }

    @keyframes pulse {
        0%, 100% { transform: scale(1); }
        50% { transform: scale(1.05); }
    }

    /* Quick Stats */
    .quick-stats {
        display: grid;
        grid-template-columns: repeat(4, 1fr);
        gap: 15px;
        margin: 20px 0;
    }

    .stat-item {
        text-align: center;
        background: white;
        padding: 20px;
        border-radius: 15px;
        box-shadow: 0 4px 15px rgba(0,0,0,0.1);
    }

    .stat-value {
        display: block;
        font-size: 24px;
        font-weight: bold;
        color: #667eea;
    }

    .stat-label {
        display: block;
        font-size: 12px;
        color: #666;
        margin-top: 5px;
    }

    /* Game Timer Large */
    .game-timer-large {
        text-align: center;
    }

    .timer-display {
        font-size: 36px;
        font-weight: bold;
        color: #ff6b6b;
        background: white;
        padding: 15px 30px;
        border-radius: 20px;
        box-shadow: 0 4px 15px rgba(0,0,0,0.1);
        display: inline-block;
    }

    .timer-label {
        font-size: 14px;
        color: #666;
        margin-top: 8px;
    }

    /* Difficulty Selector */
    .difficulty-selector {
        margin: 25px 0;
        padding: 20px;
        background: #f8f9fa;
        border-radius: 15px;
    }

    .difficulty-desc {
        color: #667eea;
        font-weight: bold;
        margin-left: 10px;
    }

    .slider {
        width: 100%;
        height: 8px;
        border-radius: 5px;
        background: #ddd;
        outline: none;
        margin: 15px 0;
    }

    .difficulty-range {
        display: flex;
        justify-content: space-between;
        font-size: 12px;
        color: #666;
    }

    /* Records Grid */
    .records-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
        gap: 15px;
        margin: 20px 0;
    }

    .record-card {
        background: white;
        padding: 20px;
        border-radius: 15px;
        text-align: center;
        box-shadow: 0 4px 15px rgba(0,0,0,0.1);
        transition: transform 0.2s;
    }

    .record-card:hover {
        transform: translateY(-5px);
    }

    .record-level {
        font-weight: bold;
        color: #667eea;
        margin-bottom: 10px;
    }

    /* Level Stats */
    .level-stats-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
        gap: 15px;
        margin: 20px 0;
    }

    .level-stat-card {
        background: white;
        padding: 20px;
        border-radius: 15px;
        box-shadow: 0 4px 15px rgba(0,0,0,0.1);
    }

    .level-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 10px;
    }

    .win-rate {
        background: #28a745;
        color: white;
        padding: 4px 8px;
        border-radius: 10px;
        font-size: 12px;
        font-weight: bold;
    }

    /* Leaderboard */
    .leaderboard {
        background: white;
        padding: 30px;
        border-radius: 20px;
        margin: 30px 0;
        box-shadow: 0 6px 25px rgba(0,0,0,0.1);
    }

    .leaderboard-item {
        display: grid;
        grid-template-columns: 50px 1fr 100px;
        align-items: center;
        padding: 15px;
        border-bottom: 1px solid #eee;
        transition: background 0.2s;
    }

    .leaderboard-item:hover {
        background: #f8f9fa;
    }

    .leaderboard-item.current-user {
        background: linear-gradient(135deg, #667eea20, #764ba220);
        font-weight: bold;
        border-radius: 10px;
    }

    .rank {
        font-size: 18px;
        font-weight: bold;
        color: #667eea;
    }

    .player-stats {
        font-size: 12px;
        color: #666;
        display: block;
    }

    .player-elo {
        font-weight: bold;
        color: #28a745;
    }

    /* Promotion Notification */
    .promotion-notification {
        position: fixed;
        top: 20px;
        right: 20px;
        background: #667eea;
        color: white;
        padding: 12px 20px;
        border-radius: 15px;
        box-shadow: 0 4px 15px rgba(0,0,0,0.2);
        z-index: 999;
        animation: slideInRight 0.3s ease;
    }

    @keyframes slideInRight {
        from {
            transform: translateX(100px);
            opacity: 0;
        }
        to {
            transform: translateX(0);
            opacity: 1;
        }
    }

    /* Buttons */
    .btn {
        padding: 12px 24px;
        border: none;
        border-radius: 25px;
        font-size: 16px;
        font-weight: 600;
        cursor: pointer;
        transition: all 0.3s;
        text-decoration: none;
        display: inline-block;
    }

    .btn-large {
        padding: 18px 36px;
        font-size: 18px;
    }

    .btn-primary {
        background: linear-gradient(135deg, #667eea, #764ba2);
        color: white;
    }

    .btn-primary:hover {
        transform: translateY(-2px);
        box-shadow: 0 8px 25px rgba(102, 126, 234, 0.3);
    }

    .btn-success {
        background: linear-gradient(135deg, #56ab2f, #a8e6cf);
        color: white;
    }

    .btn-success:hover {
        transform: translateY(-2px);
        box-shadow: 0 8px 25px rgba(86, 171, 47, 0.3);
    }

    .btn-secondary {
        background: linear-gradient(135deg, #bdc3c7, #2c3e50);
        color: white;
    }

    .btn:disabled {
        opacity: 0.6;
        cursor: not-allowed;
        transform: none;
    }

    /* Panels */
    .stats-panel, .user-setup, .main-menu, .game-area {
        background: white;
        padding: 30px;
        border-radius: 20px;
        margin: 30px 0;
        box-shadow: 0 6px 25px rgba(0,0,0,0.1);
    }

    .input {
        width: 100%;
        padding: 15px;
        border: 2px solid #ddd;
        border-radius: 15px;
        font-size: 16px;
        margin: 10px 0;
    }

    .input:focus {
        border-color: #667eea;
        outline: none;
    }

    .loading, .error {
        padding: 15px;
        border-radius: 10px;
        margin: 20px 0;
        text-align: center;
        font-weight: bold;
    }

    .loading {
        background: #e3f2fd;
        color: #1976d2;
    }

    .error {
        background: #ffebee;
        color: #c62828;
    }

    /* Responsive Design */
    @media (max-width: 768px) {
        .quick-stats {
            grid-template-columns: repeat(2, 1fr);
        }
        
        .records-grid, .level-stats-grid {
            grid-template-columns: 1fr;
        }
    }
</style>