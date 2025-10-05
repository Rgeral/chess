<script lang="ts">
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	onMount(() => goto('/'));
</script>

<div class="p-6 text-center text-muted-foreground">Redirecting…</div>
