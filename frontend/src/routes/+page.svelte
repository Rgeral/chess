<script lang="ts">
  import { gameStore, gameActions } from '$lib/stores/gameStore';
  import { ChessService } from '$lib/services/chessService';
  import { goto } from '$app/navigation';
  import type { UserProfile } from '$lib/types/chess';
  let username: string = '';

  /**
   * Creates a new user, loads profile, then redirects to the dashboard
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
      // Load profile to keep experience consistent
      try {
        const profile: UserProfile = await ChessService.getUserProfile(user.id);
        gameActions.setUserProfile(profile);
      } catch (e) { if (import.meta.env.DEV) console.error(e); }
      // Leaderboard is local on dashboard; skip storing here
      // Redirect to dashboard
      goto('/dashboard');
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : String(err);
      gameActions.setError(`Failed to create profile: ${message}`);
      if (import.meta.env.DEV) console.error(err);
    } finally {
      gameActions.setLoading(false);
    }
  }
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

        <!-- Username input -->
        <div class="space-y-4">
          <div class="relative">
            <input
              type="text"
              placeholder="Username"
              bind:value={username}
              maxlength="20"
              class="h-12 w-full text-center text-lg bg-input border border-border-50 rounded-md focus:outline-none focus:ring-2 focus:ring-primary placeholder:text-muted-foreground"
            />
            {#if username}
              <div class="absolute right-3 top-1/2 -translate-y-1/2 text-primary">♔</div>
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