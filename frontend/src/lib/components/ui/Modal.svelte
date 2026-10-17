<script lang="ts">
	export let title: string;
	export let open = false;
	export let onClose: (() => void) | null = null;

	function onKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape' && onClose) onClose();
	}
</script>

<svelte:window on:keydown={open ? onKeydown : null} />

{#if open}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4">
		<div
			class="card w-full max-w-sm rounded-[20px] p-6 shadow-2xl"
			role="dialog"
			aria-modal="true"
			aria-label={title}
		>
			<h2 class="m-0 mb-3 text-2xl font-bold tracking-tight">{title}</h2>
			<slot />
		</div>
	</div>
{/if}
