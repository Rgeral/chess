<script lang="ts">
	export let name:
		| 'home'
		| 'stats'
		| 'trophy'
		| 'star'
		| 'learn'
		| 'arrow-right'
		| 'arrow-left'
		| 'play'
		| 'flag'
		| 'bot'
		| 'logout';
	export let size = 20;

	const paths: Record<string, string> = {
		home: 'M3 11l9-7 9 7v9a1 1 0 0 1-1 1h-5v-6h-6v6H4a1 1 0 0 1-1-1z',
		stats: 'M4 20V10M10 20V4M16 20v-7M22 20H2',
		trophy: 'M8 21h8M12 17v4M7 4h10v5a5 5 0 0 1-10 0zM7 6H4a3 3 0 0 0 3 4M17 6h3a3 3 0 0 1-3 4',
		star: 'M12 3l2.7 5.6 6.1.9-4.4 4.3 1 6.1L12 17l-5.4 2.9 1-6.1L3.2 9.5l6.1-.9z',
		learn: 'M3 7l9-4 9 4-9 4zM7 9v5c0 2 2.5 3 5 3s5-1 5-3V9',
		'arrow-right': 'M5 12h14M13 6l6 6-6 6',
		'arrow-left': 'M19 12H5M11 6l-6 6 6 6',
		play: 'M7 4l13 8-13 8z',
		flag: 'M5 21V4M5 4h11l-2 4 2 4H5',
		logout: 'M9 21H5a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1h4M16 17l5-5-5-5M21 12H9',
		bot: 'M5 10a3 3 0 0 1 3-3h8a3 3 0 0 1 3 3v6a3 3 0 0 1-3 3H8a3 3 0 0 1-3-3zM12 3v4M9 12h.01M15 12h.01M9 16h6'
	};
</script>

<svg
	aria-hidden="true"
	width={size}
	height={size}
	viewBox="0 0 24 24"
	fill={name === 'play' ? 'currentColor' : 'none'}
	stroke="currentColor"
	stroke-width="1.9"
	stroke-linecap="round"
	stroke-linejoin="round"
>
	<path d={paths[name]} />
</svg>
