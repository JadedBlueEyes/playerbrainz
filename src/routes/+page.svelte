<script lang="ts">
    import type { PageData } from './$houdini'
    import Player from './Player.svelte';
    import Track from "./Track.svelte";
    import { getContext } from 'svelte';

    let getPlayer: any = getContext('player');

    interface Props {
        data: PageData
    }

    let { data } = $props()
    let { GetFlacTracks } = $derived(data);

    let dir: "prev" | "next" = "next"

    async function prevPage() {
        console.log($GetFlacTracks.data?.tracks.pageInfo.startCursor)
        await GetFlacTracks.fetch({ variables: { cursor: $GetFlacTracks.data?.tracks.pageInfo.startCursor, order: "DESC" } })
        dir = "prev"
    }

    async function nextPage() {
        console.log($GetFlacTracks.data?.tracks.pageInfo.endCursor)
        // console.log($GetFlacTracks.data?.tracks.pageInfo)
        await GetFlacTracks.fetch({ variables: { cursor: $GetFlacTracks.data?.tracks.pageInfo.endCursor, order: "ASC" } })
        dir = "next"
    }

</script>

<Player></Player>

<div class="tracks">
    {#if $GetFlacTracks.data?.tracks?.nodes}
        {#each $GetFlacTracks.data.tracks.nodes as track (track.recordingId)}
            <Track track={track}/>
        {/each}
    {/if}
</div>
<button
    disabled={!$GetFlacTracks.data?.tracks.pageInfo.hasPreviousPage}
    onclick={prevPage}
> Prev </button>

<button
    disabled={!$GetFlacTracks.data?.tracks.pageInfo.hasNextPage}
    onclick={nextPage}
> Next </button>
<style>
    .tracks {
        display: flex;
        flex-flow: column wrap;
        flex-wrap: wrap;
    }
</style>
