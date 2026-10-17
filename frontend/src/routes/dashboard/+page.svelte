<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { goto } from '$app/navigation';
	import { gameStore, gameActions } from '$lib/stores/gameStore';
	import { ChessService } from '$lib/services/chessService';
	import { restoreSession } from '$lib/restore';
	import { toBackendLevel, levelName, UI_LEVELS } from '$lib/difficulty';
	import { MOCK_RECENT_GAMES, MOCK_ACHIEVEMENTS, MOCK_LEARNING } from '$lib/mock';
	import Sidebar from '$lib/components/dashboard/Sidebar.svelte';
	import GameScreen from '$lib/components/game/GameScreen.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import Icon from '$lib/components/ui/Icon.svelte';
	import type { User } from '$lib/types/chess';

	type View = 'home' | 'stats' | 'leaderboard';

	let ready = false;
	let view: View = 'home';
	let level = 5; // niveau UI (1-10), converti en niveau Stockfish au lancement
	let leaderboard: User[] = [];
	let confirmResign = false;
	let resigned = false;
	let gameOverOpen = false;

	$: user = $gameStore.user;
	$: profile = $gameStore.userProfile;
	$: elo = profile?.user.estimatedElo ?? user?.estimatedElo ?? 800;
	$: totalGames = profile?.user.totalGames ?? 0;
	$: winRate = getWinRate(profile?.user.gamesWon ?? 0, totalGames);
	$: game = $gameStore.currentGame;

	onMount(async () => {
		if (!(await restoreSession())) {
			await goto('/');
			return;
		}
		ready = true;
		await Promise.all([loadUserProfile(), loadLeaderboard()]);
	});

	onDestroy(() => {
		gameActions.stopTimer();
	});

	function getWinRate(gamesWon: number, total: number): number {
		return total === 0 ? 0 : Math.round((gamesWon / total) * 100);
	}

	async function loadUserProfile(): Promise<void> {
		if (!$gameStore.user) return;
		try {
			gameActions.setUserProfile(await ChessService.getUserProfile($gameStore.user.id));
		} catch (error) {
			if (import.meta.env.DEV) console.error('Profile loading error:', error);
		}
	}

	async function loadLeaderboard(): Promise<void> {
		try {
			leaderboard = await ChessService.getLeaderboard(10);
		} catch (error) {
			if (import.meta.env.DEV) console.error('Leaderboard loading error:', error);
		}
	}

	async function startNewGame(): Promise<void> {
		if (!$gameStore.user) return;
		gameActions.setLoading(true);
		gameActions.setError(null);
		try {
			const created = await ChessService.createGame($gameStore.user.id, toBackendLevel(level));
			resigned = false;
			gameActions.setCurrentGame(created);
			gameActions.startTimer();
		} catch (error: unknown) {
			const message = error instanceof Error ? error.message : String(error);
			gameActions.setError(`Impossible de lancer la partie : ${message}`);
		} finally {
			gameActions.setLoading(false);
		}
	}

	function backToMenu(): void {
		gameOverOpen = false;
		gameActions.clearGame();
		loadUserProfile();
		loadLeaderboard();
	}

	async function resignGame(): Promise<void> {
		confirmResign = false;
		if (!$gameStore.currentGame || $gameStore.currentGame.status === 'finished') return;
		resigned = true;
		gameActions.setCurrentGame({
			...$gameStore.currentGame,
			status: 'finished',
			result: 'black',
			endTime: new Date().toISOString()
		});
		gameActions.stopTimer();
	}

	function onNavSelect(id: string): void {
		if (id === 'stats' || id === 'leaderboard' || id === 'home') {
			view = id;
			return;
		}
		view = 'home';
		// Succès / Apprentissage : sections de l'accueil
		requestAnimationFrame(() =>
			document.getElementById(id)?.scrollIntoView({ behavior: 'smooth' })
		);
	}

	// Fin de partie : on ouvre la modale une fois, puis on rafraîchit les stats
	let gameOverHandled = false;
	$: if (game?.status === 'finished' && !gameOverHandled) {
		gameOverHandled = true;
		gameOverOpen = true;
		loadUserProfile();
		loadLeaderboard();
	}
	$: if (game?.status === 'active') gameOverHandled = false;

	$: outcome =
		game?.result === 'white'
			? { title: 'Victoire !', text: 'Bravo, tu as battu l’IA.' }
			: game?.result === 'black'
				? {
						title: resigned ? 'Abandon' : 'Défaite',
						text: resigned ? 'Tu as abandonné la partie.' : 'L’IA remporte cette partie.'
					}
				: { title: 'Partie nulle', text: 'Personne ne l’emporte cette fois.' };
</script>

<svelte:head>
	<title>Dashboard — ChessClub</title>
</svelte:head>

{#if !ready || !user}
	<div class="flex min-h-screen items-center justify-center text-ink-subtle">Chargement…</div>
{:else if game}
	<GameScreen
		username={user.username}
		{elo}
		on:back={backToMenu}
		on:resign={() => (confirmResign = true)}
	/>

	<Modal
		title="Abandonner la partie ?"
		open={confirmResign}
		onClose={() => (confirmResign = false)}
	>
		<p class="m-0 mb-5 text-ink-muted">L’IA gagnera cette partie.</p>
		<div class="flex justify-end gap-3">
			<button
				type="button"
				class="btn-secondary rounded-[10px] px-4 py-2.5"
				on:click={() => (confirmResign = false)}>Continuer</button
			>
			<button
				type="button"
				class="rounded-[10px] border border-danger-line bg-danger-bg px-4 py-2.5 font-semibold text-danger"
				on:click={resignGame}>Abandonner</button
			>
		</div>
	</Modal>

	<Modal title={outcome.title} open={gameOverOpen} onClose={() => (gameOverOpen = false)}>
		<p class="m-0 mb-4 text-ink-muted">{outcome.text}</p>
		<dl class="m-0 mb-5 grid grid-cols-2 gap-3">
			<div class="rounded-xl bg-surface-2 p-3">
				<dt class="text-[13px] text-ink-subtle">Temps</dt>
				<dd class="m-0 font-mono text-xl font-bold">
					{gameActions.formatTime($gameStore.elapsedTime)}
				</dd>
			</div>
			<div class="rounded-xl bg-surface-2 p-3">
				<dt class="text-[13px] text-ink-subtle">Coups</dt>
				<dd class="m-0 font-mono text-xl font-bold">{game.movesCount}</dd>
			</div>
		</dl>
		<div class="flex justify-end gap-3">
			<button
				type="button"
				class="btn-secondary rounded-[10px] px-4 py-2.5"
				on:click={() => (gameOverOpen = false)}>Voir l’échiquier</button
			>
			<button type="button" class="btn-primary rounded-[10px] px-4 py-2.5" on:click={backToMenu}
				>Retour au menu</button
			>
		</div>
	</Modal>
{:else}
	<div class="flex min-h-screen flex-wrap">
		<Sidebar
			active={view}
			username={user.username}
			{elo}
			on:select={(e) => onNavSelect(e.detail)}
		/>

		<main
			class="box-border flex min-w-0 flex-[999_1_560px] flex-col gap-6 px-5 pt-8 pb-12 sm:px-10"
		>
			<div>
				<h1 class="m-0 text-[40px] font-bold tracking-[-0.03em]">
					{view === 'stats'
						? 'Statistiques'
						: view === 'leaderboard'
							? 'Classement'
							: `Bienvenue, ${user.username}`}
				</h1>
				<p class="m-0 text-ink-subtle">
					{view === 'home'
						? 'Prêt pour ta prochaine partie ?'
						: 'Tes résultats, niveau par niveau.'}
				</p>
			</div>

			{#if $gameStore.error}
				<p role="alert" class="m-0 rounded-xl bg-danger-bg px-4 py-3 text-sm text-danger">
					{$gameStore.error}
				</p>
			{/if}

			{#if view === 'home'}
				<section class="card flex flex-wrap items-center gap-8 rounded-[22px] p-7">
					<div class="flex min-w-0 flex-[1_1_420px] flex-col gap-[18px]">
						<div class="flex flex-wrap items-baseline justify-between gap-2">
							<h2 class="m-0 text-[26px] font-bold tracking-tight">Nouvelle partie contre l'IA</h2>
							<span class="text-sm text-ink-muted"
								>Difficulté : <strong class="text-accent">{levelName(level)}</strong></span
							>
						</div>
						<div role="group" aria-label="Niveau de l'IA" class="grid grid-cols-10 gap-1.5">
							{#each Array.from({ length: UI_LEVELS }, (_, i) => i + 1) as n}
								<button
									type="button"
									aria-label="Niveau {n}"
									aria-pressed={n === level}
									on:click={() => (level = n)}
									class="h-11 rounded-[10px] font-mono font-bold {n === level
										? 'border-0 bg-accent text-bg'
										: n < level
											? 'border-0 bg-surface-4 text-ink'
											: 'border border-line-strong bg-transparent text-ink-muted'}">{n}</button
								>
							{/each}
						</div>
						<div class="flex justify-between text-[13px] text-ink-subtle">
							<span>Débutant</span><span>Intermédiaire</span><span>Expert</span>
						</div>
					</div>
					<div class="flex flex-[0_1_280px] flex-col gap-2.5">
						<button
							type="button"
							class="btn-primary h-14 rounded-[14px] text-[17px]"
							disabled={$gameStore.loading}
							on:click={startNewGame}
						>
							<Icon name="play" size={18} />{$gameStore.loading
								? 'Lancement…'
								: `Commencer · Niveau ${level}`}
						</button>
						<span class="text-center text-[13px] text-ink-subtle">Tu joues les blancs</span>
					</div>
				</section>

				<div class="grid gap-3.5" style="grid-template-columns:repeat(auto-fit,minmax(200px,1fr))">
					<div class="card flex flex-col gap-1.5 rounded-2xl px-5 py-[18px]">
						<span class="text-sm text-ink-muted">Parties jouées</span>
						<span class="font-mono text-[34px] font-bold">{totalGames}</span>
					</div>
					<div class="card flex flex-col gap-1.5 rounded-2xl px-5 py-[18px]">
						<span class="text-sm text-ink-muted">Taux de victoire</span>
						<span class="font-mono text-[34px] font-bold">{winRate} %</span>
					</div>
					<div class="card flex flex-col gap-1.5 rounded-2xl px-5 py-[18px]">
						<span class="text-sm text-ink-muted">Classement</span>
						<span class="font-mono text-[34px] font-bold">{elo}</span>
					</div>
					<div
						class="flex flex-col gap-1.5 rounded-2xl border border-dashed border-line-strong bg-surface px-5 py-[18px]"
					>
						<span class="text-sm text-ink-muted">Progression</span>
						<span class="text-sm text-ink-subtle"
							>Ta courbe apparaîtra après ta première partie.</span
						>
					</div>
				</div>

				<div class="flex flex-wrap items-start gap-5">
					<!-- Données factices : voir lib/mock.ts -->
					<section
						class="card flex min-w-0 flex-[1_1_420px] flex-col gap-1.5 rounded-[20px] p-[22px]"
					>
						<div class="mb-2 flex items-center justify-between">
							<h2 class="m-0 text-[22px] font-bold">Parties récentes</h2>
							<button
								type="button"
								class="border-0 bg-transparent text-sm text-ink-muted hover:text-accent"
								on:click={() => (view = 'stats')}>Tout voir →</button
							>
						</div>
						{#each MOCK_RECENT_GAMES as g}
							<div class="flex items-center gap-3.5 rounded-xl bg-surface-2 p-3">
								<span
									class="flex h-10 w-10 flex-none items-center justify-center rounded-[10px] bg-line font-mono font-bold"
									>{g.level}</span
								>
								<span class="flex min-w-0 flex-1 flex-col leading-snug">
									<span class="font-semibold">IA Niveau {g.level}</span>
									<span class="text-[13px] text-ink-subtle">{g.moves} coups · {g.minutes} min</span>
								</span>
								<span
									class="inline-block min-w-[72px] rounded-full px-2.5 py-1 text-center text-[13px] font-semibold {g.result ===
									'win'
										? 'bg-success-bg text-success'
										: 'bg-danger-bg text-danger'}"
									>{g.result === 'win' ? 'Victoire' : 'Défaite'}</span
								>
								<span
									class="rounded-lg border border-line-strong px-3 py-2 text-[13px] font-medium text-ink-subtle"
									title="Analyse bientôt disponible">Analyser</span
								>
							</div>
						{/each}
					</section>

					<section
						id="achievements"
						class="card flex min-w-0 flex-[1_1_320px] flex-col gap-3 rounded-[20px] p-[22px]"
					>
						<div class="flex items-center justify-between">
							<h2 class="m-0 text-[22px] font-bold">Succès</h2>
							<span class="font-mono text-[13px] text-ink-subtle"
								>{MOCK_ACHIEVEMENTS.filter((a) => a.unlocked).length} / {MOCK_ACHIEVEMENTS.length}</span
							>
						</div>
						<div class="grid grid-cols-2 gap-2.5">
							{#each MOCK_ACHIEVEMENTS as a}
								<div
									class="flex flex-col gap-1.5 rounded-[14px] p-3.5 {a.unlocked
										? 'bg-surface-3 text-accent'
										: 'border border-dashed border-line-strong text-ink-faint'}"
								>
									<span class="text-[28px] leading-none" aria-hidden="true">{a.glyph}</span>
									<span class="text-sm font-semibold">{a.name}</span>
									<span class="text-xs text-ink-subtle"
										>{a.unlocked ? 'Débloqué' : 'Verrouillé'}</span
									>
								</div>
							{/each}
						</div>
					</section>
				</div>

				<section id="learning" class="flex flex-col gap-3.5">
					<h2 class="m-0 text-[22px] font-bold">Apprentissage</h2>
					<div
						class="grid gap-3.5"
						style="grid-template-columns:repeat(auto-fit,minmax(240px,1fr))"
					>
						{#each MOCK_LEARNING as item}
							<div class="card flex items-center gap-3.5 rounded-2xl p-[18px]">
								<span
									class="flex h-[46px] w-[46px] flex-none items-center justify-center rounded-xl bg-surface-3 text-[26px] leading-none text-accent"
									aria-hidden="true">{item.glyph}</span
								>
								<span class="flex flex-col leading-snug">
									<span class="font-semibold">{item.title}</span>
									<span class="text-[13px] text-ink-subtle">{item.subtitle}</span>
								</span>
							</div>
						{/each}
					</div>
				</section>
			{:else if view === 'stats'}
				{#if profile && profile.levelStats.length > 0}
					<div
						class="grid gap-3.5"
						style="grid-template-columns:repeat(auto-fit,minmax(240px,1fr))"
					>
						{#each profile.levelStats as stats}
							<div class="card rounded-2xl p-5">
								<div class="mb-2 flex items-center justify-between">
									<strong class="font-display text-lg">Niveau {stats.difficulty}</strong>
									<span class="rounded-full bg-surface-3 px-2.5 py-1 font-mono text-xs text-accent"
										>{getWinRate(stats.gamesWon, stats.gamesPlayed)} %</span
									>
								</div>
								<div class="text-sm text-ink-muted">
									{stats.gamesPlayed} parties · {stats.gamesWon} victoires
								</div>
								<div class="text-sm text-ink-subtle">
									Durée moy. {gameActions.formatTime(stats.averageTimeSeconds)} · {stats.averageMoves}
									coups moy.
								</div>
							</div>
						{/each}
					</div>
				{:else}
					<div
						class="rounded-2xl border border-dashed border-line-strong p-8 text-center text-ink-subtle"
					>
						Tes statistiques détaillées apparaîtront après ta première partie.
					</div>
				{/if}

				{#if profile && profile.records.length > 0}
					<h2 class="m-0 text-[22px] font-bold">Records</h2>
					<div
						class="grid gap-3.5"
						style="grid-template-columns:repeat(auto-fit,minmax(240px,1fr))"
					>
						{#each profile.records as record}
							<div class="card rounded-2xl p-5">
								<div class="font-display font-bold">Niveau {record.difficulty}</div>
								<div class="font-mono text-sm text-ink-muted">
									{gameActions.formatTime(record.bestTimeSeconds)} · {record.movesCount} coups
								</div>
							</div>
						{/each}
					</div>
				{/if}
			{:else}
				<section class="card rounded-[20px] p-[22px]">
					{#if leaderboard.length === 0}
						<p class="m-0 text-center text-ink-subtle">Aucun joueur classé pour l’instant.</p>
					{:else}
						<ol class="m-0 list-none p-0">
							{#each leaderboard as player, index}
								<li
									class="flex items-center justify-between gap-4 border-t border-line py-3 first:border-t-0 {player.username ===
									user.username
										? 'text-accent'
										: ''}"
								>
									<div class="flex items-center gap-4">
										<span class="w-8 font-mono font-bold text-accent">#{index + 1}</span>
										<div>
											<div class="font-semibold">{player.username}</div>
											<div class="text-xs text-ink-subtle">
												{player.gamesWon}/{player.totalGames} victoires
												{#if (player.currentStreak ?? 0) > 0}· série de {player.currentStreak}{/if}
											</div>
										</div>
									</div>
									<div class="font-mono font-bold">{player.estimatedElo} Elo</div>
								</li>
							{/each}
						</ol>
					{/if}
				</section>
			{/if}
		</main>
	</div>
{/if}
