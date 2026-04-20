<script lang="ts">
	import favicon from '$lib/assets/favicon.svg';
	import { setContext } from 'svelte';

	let { children } = $props();

	let player = $state({});
	setContext('player', () => player);

</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

{@render children()}
