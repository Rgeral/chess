<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import Logo from '$lib/components/ui/Logo.svelte';
	import Icon from '$lib/components/ui/Icon.svelte';
	import { clearSession } from '$lib/session';

	export let active: string;
	export let username: string;
	export let elo: number;

	function logout(): void {
		clearSession();
		// rechargement complet : vide aussi le store en mémoire
		window.location.href = '/';
	}

	const dispatch = createEventDispatcher<{ select: string }>();

	const items = [
		{ id: 'home', label: 'Accueil', icon: 'home' },
		{ id: 'stats', label: 'Statistiques', icon: 'stats' },
		{ id: 'leaderboard', label: 'Classement', icon: 'trophy' }
	] as const;
</script>

<aside
	class="box-border flex max-w-[260px] flex-[1_1_240px] flex-col gap-7 border-r md:sticky md:top-0 md:h-screen md:self-start md:overflow-y-auto border-line px-4 py-6 max-md:max-w-none max-md:border-r-0 max-md:border-b"
>
	<a href="/dashboard" class="px-2" on:click|preventDefault={() => dispatch('select', 'home')}>
		<Logo size={32} />
	</a>

	<nav aria-label="Menu" class="flex flex-col gap-1">
		{#each items as item}
			<button
				type="button"
				aria-current={active === item.id ? 'page' : undefined}
				on:click={() => dispatch('select', item.id)}
				class="flex items-center gap-3 rounded-[10px] border-0 px-3 py-[11px] text-left font-medium {active ===
				item.id
					? 'bg-surface-3 text-accent'
					: 'bg-transparent text-ink-muted hover:bg-surface-2'}"
			>
				<Icon name={item.icon} />{item.label}
			</button>
		{/each}
	</nav>

	<div class="mt-auto flex flex-col gap-2">
		<div class="flex items-center gap-2.5 rounded-xl bg-surface p-3">
			<span
				class="flex h-9 w-9 items-center justify-center rounded-[10px] bg-accent font-bold text-bg"
				>{username.charAt(0).toUpperCase()}</span
			>
			<span class="flex min-w-0 flex-col leading-tight">
				<span class="truncate font-semibold">{username}</span>
				<span class="font-mono text-[13px] text-ink-subtle">{elo} Elo</span>
			</span>
		</div>
		<button
			type="button"
			on:click={logout}
			class="flex items-center justify-center gap-2 rounded-[10px] border border-line-strong bg-transparent px-3 py-2.5 font-medium text-ink-muted hover:bg-surface-2 hover:text-danger"
		>
			<Icon name="logout" size={18} />Se déconnecter
		</button>
	</div>
</aside>
