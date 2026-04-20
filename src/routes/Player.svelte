<script lang="ts">
    import { getContext } from "svelte";
    import { graphql } from '$houdini';

    let getPlayer: any = getContext('player');
    let player = getPlayer();

    let trackQuery = graphql(`
        query GetTrackDetails($id: Int!) {
            tracks(filters: { id: { eq: $id } }) {
                nodes {
                    id
                    title
                    artist
                    album
                    durationSecs
                    genre
                }
            }
        }
    `);

    $effect(() => {
        if (player.currentTrackId) {
            trackQuery.fetch({ variables: { id: player.currentTrackId } });
        }
    });
</script>

{#if $trackQuery.fetching}
    <p>Loading track context...</p>
{:else if $trackQuery.data?.tracks?.nodes?.[0]}
    {@const track = $trackQuery.data.tracks.nodes[0]}
    <div class="player">
        <div class="track-info">
        <h2>Currently Playing: {track?.title}</h2>
        <p>Artist: {track?.artist}</p>
        <p>Album: {track?.album}</p>
        <p>Genre: {track?.genre}</p>
        </div>
        <div class="track">
        <audio controls src="http://localhost:8000/track/{track.id}" autoplay bind:currentTime={player.currentTime} bind:paused={player.paused}></audio>
        </div>
    </div>
{:else}
    <p>Nothing playing.</p>
{/if}

<style>
    .player {
        padding: 1rem;
        background: #f0f0f0;
        border-radius: 8px;
        margin-bottom: 1rem;
    }
</style>
