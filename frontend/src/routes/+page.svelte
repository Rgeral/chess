<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { gameStore, gameActions } from '$lib/stores/gameStore';
	import { ChessService } from '$lib/services/chessService';
	import { saveSession } from '$lib/session';
	import { restoreSession } from '$lib/restore';
	import Logo from '$lib/components/ui/Logo.svelte';
	import Icon from '$lib/components/ui/Icon.svelte';

	const START_BOARD = 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR';
	const GLYPHS: Record<string, string> = {
		k: '♚',
		q: '♛',
		r: '♜',
		b: '♝',
		n: '♞',
		p: '♟'
	};
	const squares = START_BOARD.split('/').flatMap((row, r) => {
		const cells = [...row].flatMap((ch) => (/\d/.test(ch) ? Array(Number(ch)).fill('') : [ch]));
		return cells.map((ch, c) => ({
			light: (r + c) % 2 === 0,
			glyph: ch ? GLYPHS[ch.toLowerCase()] : '',
			white: !!ch && ch === ch.toUpperCase()
		}));
	});

	const USERNAME_RE = /^[A-Za-z0-9_]{3,20}$/;
	const features = [
		{ glyph: '♞', text: "10 niveaux d'IA, de Débutant à Expert" },
		{ glyph: '♜', text: 'Statistiques, classement et succès' },
		{ glyph: '♝', text: 'Tutoriels et analyse de tes parties' }
	];

	let username = '';
	let touched = false;

	$: trimmed = username.trim();
	$: valid = USERNAME_RE.test(trimmed);

	onMount(async () => {
		if (await restoreSession()) await goto('/dashboard');
	});

	async function createUser() {
		touched = true;
		if (!valid || $gameStore.loading) return;
		gameActions.setLoading(true);
		gameActions.setError(null);
		try {
			const user = await ChessService.createUser(trimmed);
			gameActions.setUser(user);
			saveSession({ id: user.id, username: user.username });
			await goto('/dashboard');
		} catch (error: unknown) {
			const message = error instanceof Error ? error.message : String(error);
			gameActions.setError(`Impossible de créer le profil : ${message}`);
		} finally {
			gameActions.setLoading(false);
		}
	}
</script>

<svelte:head>
	<title>Créer ton profil — ChessClub</title>
</svelte:head>

<div class="flex min-h-screen flex-wrap bg-bg">
	<section
		class="flex min-w-0 flex-[1_1_560px] flex-col justify-between gap-10 border-r border-line bg-bg-raised px-6 py-10 sm:px-14"
	>
		<Logo size={34} />
		<div class="flex flex-col items-start gap-8">
			<div
				role="img"
				aria-label="Échiquier en position de départ"
				class="grid aspect-square w-full max-w-[420px] -rotate-3 grid-cols-8 grid-rows-8 overflow-hidden rounded-[14px] shadow-[0_30px_80px_rgba(0,0,0,0.45)]"
			>
				{#each squares as sq}
					<div
						class="flex items-center justify-center leading-none"
						style="font-size:clamp(20px,3vw,40px);background:{sq.light
							? '#E9E4D4'
							: '#5E7464'};color:{sq.white ? '#FFFFFF' : '#14110F'};{sq.white
							? 'text-shadow:0 0 2px #14110F,0 1px 1px #14110F'
							: ''}"
					>
						{sq.glyph}
					</div>
				{/each}
			</div>
			<h2
				class="m-0 max-w-[520px] text-5xl font-extrabold leading-none tracking-[-0.035em] sm:text-[56px]"
			>
				Stratégie.<br />Tactique.<br /><span class="text-accent">Maîtrise.</span>
			</h2>
		</div>
		<p class="m-0 text-sm text-ink-subtle">
			Affronte l'IA du niveau 1 au niveau 10 et suis ta progression partie après partie.
		</p>
	</section>

	<section class="flex min-w-0 flex-[1_1_440px] items-center justify-center px-8 py-10">
		<form
			class="flex w-full max-w-[400px] flex-col gap-[22px]"
			on:submit|preventDefault={createUser}
			novalidate
		>
			<div class="flex flex-col gap-2">
				<h1 class="m-0 text-[40px] font-bold tracking-[-0.03em]">Crée ton profil</h1>
				<p class="m-0 text-ink-muted">
					Choisis un pseudo pour commencer à jouer et suivre tes progrès. Pas de mot de passe, pas
					d'e-mail.
				</p>
			</div>

			<div class="flex flex-col gap-2">
				<label for="pseudo" class="text-sm font-medium text-ink-2">Pseudo</label>
				<input
					id="pseudo"
					type="text"
					bind:value={username}
					maxlength="20"
					placeholder="ex. magnus_du_93"
					autocomplete="username"
					aria-invalid={touched && !valid}
					aria-describedby="pseudo-help"
					class="box-border h-[52px] rounded-xl border bg-surface px-4 text-[17px] text-ink outline-none placeholder:text-ink-subtle focus:border-accent {touched &&
					!valid
						? 'border-danger'
						: 'border-line-strong'}"
				/>
				<span
					id="pseudo-help"
					class="text-[13px] {touched && !valid ? 'text-danger' : 'text-ink-subtle'}"
				>
					3 à 20 caractères, lettres, chiffres et _
				</span>
			</div>

			{#if $gameStore.error}
				<p role="alert" class="m-0 rounded-xl bg-danger-bg px-4 py-3 text-sm text-danger">
					{$gameStore.error}
				</p>
			{/if}

			<button
				type="submit"
				class="btn-primary h-[54px] rounded-xl text-[17px]"
				disabled={$gameStore.loading || !valid}
			>
				{$gameStore.loading ? 'Création…' : 'Créer mon profil'}
				<Icon name="arrow-right" size={18} />
			</button>

			<ul class="m-0 flex list-none flex-col gap-3 border-t border-line p-0 pt-[18px]">
				{#each features as f}
					<li class="flex items-center gap-3 text-sm text-ink-muted">
						<span
							class="flex h-8 w-8 items-center justify-center rounded-[9px] bg-surface text-lg text-accent"
							aria-hidden="true">{f.glyph}</span
						>
						{f.text}
					</li>
				{/each}
			</ul>
		</form>
	</section>
</div>
