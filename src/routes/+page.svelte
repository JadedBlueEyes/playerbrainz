<script lang="ts">
    import type { PageData } from './$houdini'
    import Track from "./Track.svelte";

    interface Props {
        data: PageData
    }
    let { data } = $props()
    let { GetFlacTracks } = $derived(data);

    let dir: "prev" | "next" = "next"

    let nowPlaying: NonNullable<Awaited<ReturnType<PageData['GetFlacTracks']['fetch']>>['data']>['tracks']['nodes'][number] | null = $state(null)

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

    $effect(() => console.log($GetFlacTracks.data?.tracks.pageInfo))
</script>

{#if nowPlaying}
	<audio controls src="http://localhost:8000/track/{nowPlaying.id}" autoplay></audio>
{:else}
<p>Nothing playing</p>
{/if}

{#if $GetFlacTracks.data?.tracks?.nodes}
    {#each $GetFlacTracks.data.tracks.nodes as track (track.recordingId)}
        <Track track={track} onclick={() => nowPlaying = track}/>
    {/each}
{/if}

<button
    disabled={!$GetFlacTracks.data?.tracks.pageInfo.hasPreviousPage}
    onclick={prevPage}
> Prev </button>

<button
    disabled={!$GetFlacTracks.data?.tracks.pageInfo.hasNextPage}
    onclick={nextPage}
> Next </button>
