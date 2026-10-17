<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import { gameStore, gameActions } from '$lib/stores/gameStore';
	import ChessBoard from '$lib/components/ChessBoard/ChessBoard.svelte';
	import Logo from '$lib/components/ui/Logo.svelte';
	import Icon from '$lib/components/ui/Icon.svelte';
	import { fromBackendLevel, levelName } from '$lib/difficulty';
	import { capturedFromFen } from '$lib/captured';

	export let username: string;
	export let elo: number;

	const dispatch = createEventDispatcher<{ back: void; resign: void }>();

	$: game = $gameStore.currentGame;
	$: active = game?.status === 'active';
	$: uiLevel = fromBackendLevel(game?.difficulty ?? 2);
	$: captured = capturedFromFen(game?.fen ?? '');
	$: aiThinking = active && $gameStore.loading;
</script>

<div class="flex min-h-screen flex-col">
	<header
		class="flex flex-wrap items-center justify-between gap-4 border-b border-line px-5 py-[18px] sm:px-8"
	>
		<button
			type="button"
			class="btn-secondary rounded-[10px] px-3.5 py-2.5"
			on:click={() => dispatch('back')}
		>
			<Icon name="arrow-left" size={18} />Retour au menu
		</button>
		<Logo size={30} />
		<span class="flex items-center gap-2 text-sm text-ink-muted">
			<span class="h-2 w-2 rounded-full {active ? 'bg-online' : 'bg-ink-faint'}" aria-hidden="true"
			></span>
			{active ? 'Partie en cours' : 'Partie terminée'}
		</span>
	</header>

	<div
		class="mx-auto box-border flex w-full max-w-[1280px] flex-1 flex-wrap items-start gap-7 px-5 py-7 sm:px-8"
	>
		<div class="flex min-w-0 max-w-[760px] flex-[1_1_560px] flex-col gap-3">
			<div class="card flex items-center justify-between rounded-[14px] px-4 py-3">
				<span class="flex items-center gap-3">
					<span class="flex h-9 w-9 items-center justify-center rounded-[10px] bg-line text-ink-2">
						<Icon name="bot" />
					</span>
					<span class="flex flex-col leading-tight">
						<span class="font-semibold">IA Niveau {uiLevel}</span>
						<span class="text-[13px] text-ink-subtle">{levelName(uiLevel)} · noirs</span>
					</span>
				</span>
				<span class="text-[13px] text-ink-subtle" aria-live="polite">
					{aiThinking ? 'Réfléchit…' : ''}
				</span>
			</div>

			<div class="overflow-hidden rounded-[14px] shadow-[0_30px_80px_rgba(0,0,0,0.45)]">
				<ChessBoard lastMove={$gameStore.lastMove} allowMoves={!!active} />
			</div>

			<div
				class="card flex items-center justify-between rounded-[14px] px-4 py-3 {active
					? 'border-accent'
					: ''}"
			>
				<span class="flex items-center gap-3">
					<span
						class="flex h-9 w-9 items-center justify-center rounded-[10px] bg-accent font-bold text-bg"
						>{username.charAt(0).toUpperCase()}</span
					>
					<span class="flex flex-col leading-tight">
						<span class="font-semibold">{username}</span>
						<span class="text-[13px] text-ink-subtle">{elo} · blancs</span>
					</span>
				</span>
				{#if active && !aiThinking}
					<span class="text-[13px] font-semibold text-accent">À toi de jouer</span>
				{/if}
			</div>
		</div>

		<aside class="flex min-w-0 flex-[1_1_320px] flex-col gap-4">
			<div class="grid grid-cols-2 gap-3">
				<div class="card rounded-2xl px-[18px] py-4">
					<div class="text-[13px] text-ink-subtle">Temps de jeu</div>
					<div class="font-mono text-[30px] font-bold">
						{gameActions.formatTime($gameStore.elapsedTime)}
					</div>
				</div>
				<div class="card rounded-2xl px-[18px] py-4">
					<div class="text-[13px] text-ink-subtle">Coups</div>
					<div class="font-mono text-[30px] font-bold">{game?.movesCount ?? 0}</div>
				</div>
			</div>

			<section class="card flex min-h-[300px] flex-col gap-3 rounded-[18px] p-5">
				<h2 class="m-0 text-xl font-bold">Coups joués</h2>
				<div
					class="flex flex-1 flex-col items-center justify-center gap-2 rounded-xl border border-dashed border-line-strong p-6 text-center text-ink-subtle"
				>
					<span class="text-[32px] leading-none text-ink-faint" aria-hidden="true">♙</span>
					{#if (game?.movesCount ?? 0) === 0}
						Aucun coup pour l'instant.<br />Clique sur une pièce blanche pour commencer.
					{:else}
						{game?.movesCount} coups joués.<br />La liste en notation arrive bientôt.
					{/if}
				</div>
			</section>

			<section class="card flex flex-col gap-2.5 rounded-[18px] p-5">
				<h2 class="m-0 text-xl font-bold">Pièces capturées</h2>
				<div class="flex justify-between gap-3 text-sm text-ink-subtle">
					<span>Par toi</span>
					<span
						class="text-lg leading-none tracking-wider text-ink"
						aria-label="Pièces prises par toi">{captured.byPlayer.join('') || '—'}</span
					>
				</div>
				<div class="flex justify-between gap-3 text-sm text-ink-subtle">
					<span>Par l'IA</span>
					<span
						class="text-lg leading-none tracking-wider text-ink"
						aria-label="Pièces prises par l'IA">{captured.byAi.join('') || '—'}</span
					>
				</div>
			</section>

			<button
				type="button"
				class="flex h-[50px] items-center justify-center gap-2 rounded-xl border border-danger-line bg-transparent font-semibold text-danger hover:bg-danger-bg disabled:opacity-40"
				disabled={!active}
				on:click={() => dispatch('resign')}
			>
				<Icon name="flag" size={18} />Abandonner
			</button>
		</aside>
	</div>
</div>
