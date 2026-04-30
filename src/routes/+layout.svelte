<script lang="ts">
	import favicon from '$lib/assets/favicon.svg';
	import { setContext, onMount } from 'svelte';
	import { Player } from '$lib/player.svelte';
	import { setClientSession } from '$houdini';

	let { data, children } = $props();

	$effect(() => {
		setClientSession({ token: data.token });
	});

	let player = new Player();

	onMount(() => {
		const stored = localStorage.getItem('playerState');
		if (stored) {
			try {
				player.load(JSON.parse(stored));
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

<style>
    :global(:root) {
        font-family: sans-serif;
        --primary-color: #024dcb; /* One of the colours from my logo :3 */
        --error-color: red;
    }
</style>
