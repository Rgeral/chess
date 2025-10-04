<script lang="ts">
    import { onDestroy } from 'svelte';
    import { gameStore, gameActions } from '$lib/stores/gameStore';
    import { ChessService } from '$lib/services/chessService';
    import ChessBoard from '$lib/components/ChessBoard/ChessBoard.svelte';
    import type { User } from '$lib/types/chess';

    let username: string = '';
    let difficulty: number = 5; // 1-20 (Stockfish)
    let gameStarted: boolean = false;
    let showStats: boolean = false;
    let showLeaderboard: boolean = false;
    let leaderboard: User[] = [];

    /**
     * Creates a new user account and loads their profile and leaderboard
     */
    async function createUser(): Promise<void> {
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
            if (import.meta.env.DEV) console.log('User created:', user);
        } catch (error: unknown) {
            const message = error instanceof Error ? error.message : String(error);
            gameActions.setError(`Failed to create user: ${message}`);
            if (import.meta.env.DEV) console.error('User creation error:', error);
        } finally {
            gameActions.setLoading(false);
        }
    }

    /**
     * Loads complete user profile with stats and records
     */
    async function loadUserProfile(): Promise<void> {
        if (!$gameStore.user) return;
        try {
            const profile = await ChessService.getUserProfile($gameStore.user.id);
            gameActions.setUserProfile(profile);
            if (import.meta.env.DEV) console.log('Profile loaded:', profile);
        } catch (error) {
            if (import.meta.env.DEV) console.error('Profile loading error:', error);
        }
    }

    /**
     * Loads leaderboard data
     */
    async function loadLeaderboard(): Promise<void> {
        try {
            leaderboard = await ChessService.getLeaderboard(10);
            if (import.meta.env.DEV) console.log('Leaderboard loaded:', leaderboard);
        } catch (error) {
            if (import.meta.env.DEV) console.error('Leaderboard loading error:', error);
        }
    }

    /**
     * Starts a new chess game against Stockfish
     */
    async function startNewGame(): Promise<void> {
        if (!$gameStore.user) return;
        gameActions.setLoading(true);
        gameActions.setError(null);
        try {
            const game = await ChessService.createGame($gameStore.user.id, difficulty);
            gameActions.setCurrentGame(game);
            gameActions.startTimer();
            gameStarted = true;
            if (import.meta.env.DEV) console.log('Game started:', game);
        } catch (error: unknown) {
            const message = error instanceof Error ? error.message : String(error);
            gameActions.setError(`Failed to start game: ${message}`);
            if (import.meta.env.DEV) console.error('Game creation error:', error);
        } finally {
            gameActions.setLoading(false);
        }
    }

    /**
     * Resets game state and returns to dashboard menu
     */
    function resetGame(): void {
        gameActions.clearGame();
        gameStarted = false;
    }

    /**
     * Calculates win rate percentage
     */
    function getWinRate(gamesWon: number, totalGames: number): number {
        if (totalGames === 0) return 0;
        return Math.round((gamesWon / totalGames) * 100);
    }

    /**
     * Resigns the current game and refreshes profile/leaderboard
     */
    async function resignGame(): Promise<void> {
        if (!$gameStore.currentGame || $gameStore.currentGame.status === 'finished') return;
        const updatedGame = {
            ...$gameStore.currentGame,
            status: 'finished',
            result: 'black',
            endTime: new Date().toISOString()
        };
        gameActions.setCurrentGame(updatedGame);
        gameActions.stopTimer();
        gameStarted = false;
        alert('You resigned! Stockfish wins.');
        await loadUserProfile();
        await loadLeaderboard();
    }

    // Show an alert when the game ends, then refresh stats
    let gameOverAlerted = false;
    $: if ($gameStore.currentGame && $gameStore.currentGame.status === 'finished' && !gameOverAlerted) {
        gameOverAlerted = true;
        let outcome = 'Game over!';
        if ($gameStore.currentGame.result === 'white') outcome = 'You won! 🏆';
        else if ($gameStore.currentGame.result === 'black') outcome = 'You lost! 😔';
        else if ($gameStore.currentGame.result === 'draw') outcome = 'Draw! 🤝';
        const time = gameActions.formatTime($gameStore.elapsedTime);
        alert(`${outcome}\n⏱️ Time: ${time}\n♟️ Moves: ${$gameStore.currentGame.movesCount}`);
        loadUserProfile();
        loadLeaderboard();
    }
    $: if ($gameStore.currentGame && $gameStore.currentGame.status === 'active') {
        gameOverAlerted = false;
    }

    onDestroy(() => {
        gameActions.stopTimer();
    });
</script>

<!-- Full-height dashboard with chess background -->
<div class="min-h-screen bg-background p-4 relative overflow-hidden">
  <!-- Background chess pattern -->
  <div class="absolute inset-0 opacity-5 pointer-events-none">
    <div class="grid grid-cols-8 h-full">
      {#each Array(64) as _, i}
        <div class={Math.floor(i / 8) % 2 === i % 2 ? 'bg-primary' : 'bg-background'}></div>
      {/each}
    </div>
  </div>

  <div class="relative z-10 max-w-6xl mx-auto">
    <!-- Header -->
    <div class="text-center mb-8">
      <div class="flex justify-center items-center space-x-3 mb-4">
        <span class="text-4xl">♔</span>
        <h1 class="text-4xl font-bold text-primary">ChessClub</h1>
        <span class="text-4xl">♕</span>
      </div>
      {#if $gameStore.user}
        <h2 class="text-2xl font-semibold text-foreground mb-2">
          Bienvenue, <span class="text-primary">{$gameStore.user.username}</span> !
        </h2>
        <p class="text-muted-foreground">Prêt pour votre prochaine partie ?</p>
      {:else}
        <div class="mx-auto max-w-md p-6 rounded-xl bg-card-80 backdrop-blur-sm border border-border-50 shadow-2xl">
          <h2 class="text-xl font-semibold text-foreground mb-2">Create Your Profile</h2>
          <p class="text-muted-foreground mb-4">Enter a username to start playing and tracking your progress.</p>
          <div class="flex gap-3">
            <input type="text" bind:value={username} maxlength="20" placeholder="Username" class="flex-1 h-11 px-3 bg-input border border-border-50 rounded-md focus:outline-none focus:ring-2 focus:ring-primary placeholder:text-muted-foreground" />
            <button class="h-11 px-5 rounded-md bg-primary text-white font-semibold disabled:opacity-60" on:click={createUser} disabled={$gameStore.loading || !username.trim()}>
              🚀 Create
            </button>
          </div>
        </div>
      {/if}
    </div>

    {#if $gameStore.user}
      {#if $gameStore.currentGame}
        <!-- Game Area -->
        <div class="space-y-6">
          <div class="p-6 rounded-xl bg-card-80 backdrop-blur-sm border border-border-50 shadow-2xl">
            <div class="flex flex-col md:flex-row md:items-center md:justify-between gap-4">
              <div>
                <h3 class="text-xl font-semibold text-foreground">♟️ vs Stockfish (Level {$gameStore.currentGame.difficulty})</h3>
                <div class="text-sm text-muted-foreground">Status: <span class="font-medium text-foreground">{$gameStore.currentGame.status}</span> • Moves: <span class="font-medium text-foreground">{$gameStore.currentGame.movesCount}</span></div>
              </div>
              <div class="text-center">
                <div class="text-2xl font-bold text-primary bg-card rounded-lg px-4 py-2 inline-block">⏱️ {gameActions.formatTime($gameStore.elapsedTime)}</div>
                <div class="text-xs text-muted-foreground mt-1">Game Time</div>
              </div>
            </div>
          </div>

          <div class="p-4 rounded-xl bg-card-80 backdrop-blur-sm border border-border-50 shadow-2xl">
            <div class="max-w-3xl mx-auto">
              <ChessBoard lastMove={$gameStore.lastMove} allowMoves={$gameStore.currentGame.status === 'active'} />
            </div>
          </div>

          <div class="flex gap-3 justify-center">
            <button class="h-11 px-5 rounded-full bg-card-80 text-foreground border border-border-50" on:click={resetGame}>🏠 Back to Menu</button>
            <button class="h-11 px-5 rounded-full bg-primary text-white disabled:opacity-60" on:click={resignGame} disabled={$gameStore.currentGame.status !== 'active'}>🚩 Resign</button>
          </div>
        </div>
      {:else}
        <!-- Dashboard Grid -->
        <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
          <!-- Left Column: User Stats -->
          <div class="space-y-6">
            <div class="p-6 rounded-xl bg-card-80 backdrop-blur-sm border border-border-50 shadow-2xl text-center space-y-4">
              <div class="text-6xl">♔</div>
              <div>
                <h3 class="text-xl font-semibold text-foreground mb-2">Vos Statistiques</h3>
                <div class="space-y-3">
                  <div class="flex justify-between items-center">
                    <span class="text-muted-foreground">Parties jouées</span>
                    <span class="px-2 py-1 rounded-md bg-secondary text-foreground font-semibold">{$gameStore.userProfile?.user.totalGames ?? 0}</span>
                  </div>
                  <div class="flex justify-between items-center">
                    <span class="text-muted-foreground">Taux de victoire</span>
                    <span class="px-2 py-1 rounded-md bg-primary text-white font-semibold">{getWinRate($gameStore.userProfile?.user.gamesWon ?? 0, $gameStore.userProfile?.user.totalGames ?? 0)}%</span>
                  </div>
                  <div class="flex justify-between items-center">
                    <span class="text-muted-foreground">Classement</span>
                    <span class="px-2 py-1 rounded-md border border-border-50 font-semibold">{$gameStore.userProfile?.user.estimatedElo ?? 800}</span>
                  </div>
                </div>
                <div class="mt-4">
                  <div class="text-sm text-muted-foreground mb-2">Progression</div>
                  <div class="h-2 w-full bg-muted rounded-full overflow-hidden">
                    <div class="h-2 bg-primary" style="width: {getWinRate($gameStore.userProfile?.user.gamesWon ?? 0, $gameStore.userProfile?.user.totalGames ?? 0)}%"></div>
                  </div>
                </div>
              </div>
            </div>

            <!-- Recent Games (placeholder) -->
            <div class="p-6 rounded-xl bg-card-80 backdrop-blur-sm border border-border-50 shadow-2xl">
              <h3 class="text-lg font-semibold text-foreground mb-4 flex items-center"><span class="mr-2">♟</span>Parties Récentes</h3>
              <div class="space-y-3">
                {#each [
                  { opponent: 'IA Niveau 3', result: 'Victoire', moves: '32', time: '15min' },
                  { opponent: 'IA Niveau 2', result: 'Défaite', moves: '28', time: '12min' },
                  { opponent: 'IA Niveau 4', result: 'Victoire', moves: '45', time: '22min' }
                ] as game, i}
                  <div class="flex justify-between items-center p-3 rounded-lg bg-card-80">
                    <div>
                      <div class="font-medium text-sm">{game.opponent}</div>
                      <div class="text-xs text-muted-foreground">{game.moves} coups • {game.time}</div>
                    </div>
                    <span class={`text-xs px-2 py-1 rounded-md ${game.result === 'Victoire' ? 'bg-primary text-white' : 'bg-card'}`}>{game.result}</span>
                  </div>
                {/each}
              </div>
            </div>
          </div>

          <!-- Center Column: Game Setup -->
          <div class="space-y-6">
            <div class="p-8 rounded-xl bg-card-80 backdrop-blur-sm border border-border-50 shadow-2xl text-center space-y-6">
              <div class="text-5xl">⚔️</div>
              <div>
                <h3 class="text-xl font-semibold text-foreground mb-2">Nouvelle Partie</h3>
                <p class="text-muted-foreground text-sm">Choisissez votre niveau de défi</p>
              </div>

              <!-- Difficulty Selector -->
              <div class="space-y-4">
                <div class="text-center">
                  <div class="text-lg font-medium text-foreground mb-2">
                    Difficulté: <span class="text-primary">{difficulty <= 5 ? 'Débutant' : difficulty <= 10 ? 'Facile' : difficulty <= 15 ? 'Moyen' : difficulty < 20 ? 'Difficile' : 'Expert'}</span>
                  </div>
                  <div class="px-2 md:px-6">
                    <input type="range" bind:value={difficulty} min="1" max="20" step="1" class="w-full" />
                  </div>
                  <div class="flex justify-between text-xs text-muted-foreground mt-2">
                    <span>Débutant</span>
                    <span>Expert</span>
                  </div>
                </div>
              </div>

              <button class="w-full h-12 text-lg font-semibold rounded-full bg-primary text-white disabled:opacity-60" on:click={startNewGame} disabled={$gameStore.loading}>
                <span class="mr-2">♔</span>
                Commencer la Partie (Level {difficulty})
              </button>
            </div>

            <!-- Quick Actions -->
            <div class="grid grid-cols-2 gap-4">
              <button class="h-16 rounded-xl border border-border-50 bg-transparent flex flex-col items-center justify-center space-y-1" on:click={() => showStats = !showStats}>
                <span class="text-xl">📊</span>
                <span class="text-sm">Statistiques</span>
              </button>
              <button class="h-16 rounded-xl border border-border-50 bg-transparent flex flex-col items-center justify-center space-y-1" on:click={() => showLeaderboard = !showLeaderboard}>
                <span class="text-xl">🏆</span>
                <span class="text-sm">Classement</span>
              </button>
            </div>
          </div>

          <!-- Right Column: Achievements & Learning -->
          <div class="space-y-6">
            <div class="p-6 rounded-xl bg-card-80 backdrop-blur-sm border border-border-50 shadow-2xl">
              <h3 class="text-lg font-semibold text-foreground mb-4 flex items-center"><span class="mr-2">🏅</span>Succès</h3>
              <div class="space-y-3">
                {#each [
                  { name: 'Première Victoire', icon: '🎯', unlocked: true },
                  { name: '10 Parties', icon: '🔥', unlocked: true },
                  { name: 'Maître Tactique', icon: '🧠', unlocked: false },
                  { name: 'Roi des Échecs', icon: '👑', unlocked: false }
                ] as achievement}
                  <div class={`flex items-center space-x-3 p-2 rounded-lg ${achievement.unlocked ? 'bg-card-80' : 'bg-card-80 opacity-50'}`}>
                    <span class="text-xl">{achievement.icon}</span>
                    <span class="text-sm font-medium">{achievement.name}</span>
                  </div>
                {/each}
              </div>
            </div>

            <div class="p-6 rounded-xl bg-card-80 backdrop-blur-sm border border-border-50 shadow-2xl">
              <h3 class="text-lg font-semibold text-foreground mb-4 flex items-center"><span class="mr-2">📚</span>Apprentissage</h3>
              <div class="space-y-3">
                {#each [
                  { title: 'Tutoriel Débutant', subtitle: 'Apprenez les bases' },
                  { title: 'Tactiques Avancées', subtitle: 'Perfectionnez votre jeu' },
                  { title: 'Analyse de Parties', subtitle: 'Étudiez vos erreurs' }
                ] as item}
                  <button class="w-full text-left p-3 rounded-lg hover:bg-card">
                    <div class="font-medium text-sm">{item.title}</div>
                    <div class="text-xs text-muted-foreground">{item.subtitle}</div>
                  </button>
                {/each}
              </div>
            </div>
          </div>
        </div>

        <!-- Toggled Panels -->
        {#if showStats && $gameStore.userProfile}
          <div class="mt-6 p-6 rounded-xl bg-card-80 backdrop-blur-sm border border-border-50 shadow-2xl">
            <h2 class="text-xl font-semibold mb-4">📊 Vos Statistiques Détaillées</h2>
            {#if $gameStore.userProfile.records.length > 0}
              <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                {#each $gameStore.userProfile.records as record}
                  <div class="p-4 rounded-lg bg-card border border-border-50">
                    <div class="font-semibold">Level {record.difficulty}</div>
                    <div class="text-sm text-muted-foreground">⏱️ {gameActions.formatTime(record.bestTimeSeconds)}</div>
                    <div class="text-sm text-muted-foreground">♟️ {record.movesCount} moves</div>
                  </div>
                {/each}
              </div>
            {/if}

            {#if $gameStore.userProfile.levelStats.length > 0}
              <div class="mt-6 grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                {#each $gameStore.userProfile.levelStats as stats}
                  <div class="p-4 rounded-lg bg-card border border-border-50">
                    <div class="flex items-center justify-between mb-2">
                      <strong>Level {stats.difficulty}</strong>
                      <span class="px-2 py-1 rounded-md bg-primary text-white text-xs">{getWinRate(stats.gamesWon, stats.gamesPlayed)}%</span>
                    </div>
                    <div class="text-sm text-muted-foreground">{stats.gamesPlayed} games • {stats.gamesWon} wins</div>
                    <div class="text-sm text-muted-foreground">⏱️ Avg: {gameActions.formatTime(stats.averageTimeSeconds)}</div>
                    <div class="text-sm text-muted-foreground">♟️ Avg: {stats.averageMoves} moves</div>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        {/if}

        {#if showLeaderboard && leaderboard.length > 0}
          <div class="mt-6 p-6 rounded-xl bg-card-80 backdrop-blur-sm border border-border-50 shadow-2xl">
            <h2 class="text-xl font-semibold mb-4">🏆 Top Players</h2>
            <div>
              {#each leaderboard as player, index}
                <div class="py-3 flex items-center justify-between border-t border-border-50 first:border-t-0">
                  <div class="flex items-center gap-4">
                    <div class="w-8 text-primary font-bold">#{index + 1}</div>
                    <div>
                      <div class="font-semibold">{player.username}</div>
                      <div class="text-xs text-muted-foreground">{player.gamesWon}/{player.totalGames} games {#if (player.currentStreak ?? 0) > 0}• 🔥{player.currentStreak}{/if}</div>
                    </div>
                  </div>
                  <div class="font-semibold text-foreground">{player.estimatedElo} ELO</div>
                </div>
              {/each}
            </div>
          </div>
        {/if}
      {/if}
    {/if}

    <!-- Promotion banner -->
    {#if $gameStore.pendingPromotion?.isActive}
      <div class="fixed top-4 right-4 rounded-xl bg-primary text-white px-4 py-2 shadow-2xl">🎯 Pawn promotion in progress...</div>
    {/if}

    <!-- Loading / Error -->
    {#if $gameStore.loading}
      <div class="mt-6 p-4 rounded-lg bg-card border border-border-50 text-center font-semibold">⏳ Loading...</div>
    {/if}
    {#if $gameStore.error}
      <div class="mt-6 p-4 rounded-lg bg-card border border-border-50 text-center font-semibold text-red-600">❌ {$gameStore.error}</div>
    {/if}

    <!-- Bottom decoration -->
    <div class="text-center mt-12 opacity-20 select-none">
      <div class="flex justify-center space-x-4 text-2xl">
        <span>♜</span>
        <span>♞</span>
        <span>♝</span>
        <span>♛</span>
        <span>♚</span>
        <span>♝</span>
        <span>♞</span>
        <span>♜</span>
      </div>
    </div>
  </div>
</div>
