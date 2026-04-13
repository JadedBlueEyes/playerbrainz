<script lang="ts">
    import type { PageData } from './$houdini'
    // import { graphql } from '$houdini'
    //
    //
    //
    import Track from "./Track.svelte";

    interface Props {
        data: PageData
    }
    let { data } = $props()
    // let { GetFlacTracks } = $derived(data)
    $effect(() => console.log(data))
    let { GetFlacTracks } = data;

    // let store = $derived(
    //     graphql(`query GetFlacTracks {
    //         tracks(
    //             filters: { filename: { ends_with: ".flac" } }
    //             pagination: { cursor: { limit: 1000 } }
    //         ) {
    //             # nodes contains the actual track data
    //             nodes {
    //                 filePath
    //                 # filename
    //                 title
    //                 artist
    //                 album
    //                 date
    //                 trackNumber
    //                 durationSecs
    //                 # artistIds
    //                 # workIds
    //                 recordingId
    //             }
    //             # pageInfo tells you if there are more tracks
    //             pageInfo {
    //                 hasNextPage
    //                 endCursor
    //             }
    //         }
    //     }

    //     `)
    // )

    // $effect(() => console.log($store))
    //
    let nowPlaying = $state(null)

</script>

{#if nowPlaying}
	<audio controls src="http://localhost:8000/track/{nowPlaying.id}" autoplay></audio>
{:else}
<p>Nothing playing</p>
{/if}

{#each $GetFlacTracks.data.tracks.nodes as track (track.recordingId)}
    <!-- <p>{JSON.stringify(track)}</p> -->
    <Track track={track} onclick={() => nowPlaying = track}/>

{/each}
<!-- {JSON.stringify($GetFlacTracks)} -->

<button
    disabled={!$GetFlacTracks.data.tracks.pageInfo.hasPreviousPage}
    onclick={async () => {console.log(GetFlacTracks); await GetFlacTracks.loadPreviousPage()}}

> Prev </button>
<button
    disabled={!$GetFlacTracks.data.tracks.pageInfo.hasNextPage}
    onclick={async () => {console.log(GetFlacTracks); await GetFlacTracks.loadNextPage()}}
> Next </button>
