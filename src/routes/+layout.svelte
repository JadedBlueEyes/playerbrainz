<script lang="ts">
	import favicon from '$lib/assets/favicon.svg';
	import { setContext, onMount } from 'svelte';

	let { children } = $props();

	let player = $state({} as any);

	onMount(() => {
		const stored = localStorage.getItem('playerState');
		if (stored) {
			try {
				Object.assign(player, JSON.parse(stored));
			} catch (e) {
				console.error("Failed to parse player state", e);
			}
		}
	});

	$effect(() => {
		if (typeof localStorage !== 'undefined') {
			localStorage.setItem('playerState', JSON.stringify(player));
		}
	});

	setContext('player', () => player);

</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

{@render children()}
