<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { createEventDispatcher } from 'svelte';

	export let isRunning = false;
	export let startTime: Date | null = null;

	const dispatch = createEventDispatcher();

	let elapsedSeconds = 0;
	let interval: NodeJS.Timeout | null = null;

	$: formattedTime = formatTime(elapsedSeconds);

	function formatTime(seconds: number): string {
		const mins = Math.floor(seconds / 60);
		const secs = seconds % 60;
		return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
	}

	function updateTimer() {
		if (startTime && isRunning) {
			elapsedSeconds = Math.floor((Date.now() - startTime.getTime()) / 1000);
		}
	}

	export function start() {
		if (!isRunning) {
			startTime = new Date();
			isRunning = true;
			dispatch('start', { startTime });
		}
	}

	export function stop() {
		if (isRunning) {
			isRunning = false;
			dispatch('stop', {
				endTime: new Date(),
				duration: elapsedSeconds
			});
		}
	}

	export function reset() {
		isRunning = false;
		startTime = null;
		elapsedSeconds = 0;
		dispatch('reset');
	}

	onMount(() => {
		interval = setInterval(updateTimer, 1000);
	});

	onDestroy(() => {
		if (interval) {
			clearInterval(interval);
		}
	});

	$: if (isRunning) {
		updateTimer();
	}
</script>

<div class="game-timer" class:running={isRunning}>
	<div class="timer-display">
		<span class="timer-icon">⏱️</span>
		<span class="timer-text">{formattedTime}</span>
	</div>

	{#if elapsedSeconds > 0}
		<div class="timer-info">
			<small>{elapsedSeconds} seconds</small>
		</div>
	{/if}
</div>

<style>
	.game-timer {
		display: flex;
		flex-direction: column;
		align-items: center;
		padding: 12px 20px;
		background: #f8f9fa;
		border-radius: 8px;
		border: 2px solid #e9ecef;
		transition: all 0.3s ease;
	}

	.game-timer.running {
		background: #e8f5e8;
		border-color: #28a745;
		animation: pulse 2s infinite;
	}

	@keyframes pulse {
		0%,
		100% {
			transform: scale(1);
		}
		50% {
			transform: scale(1.02);
		}
	}

	.timer-display {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.timer-icon {
		font-size: 20px;
	}

	.timer-text {
		font-size: 24px;
		font-weight: bold;
		font-family: 'Courier New', monospace;
		color: #2c3e50;
	}

	.running .timer-text {
		color: #28a745;
	}

	.timer-info {
		margin-top: 4px;
		color: #6c757d;
	}
</style>
