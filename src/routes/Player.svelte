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

                    trackMedia (filters:  {
                        usage:  {
                           eq: "FrontCover"
                        }

                    } orderBy:  {
                       visualIndex: ASC
                    } pagination:  {
                        page:  {
                           limit: 1
                        }
                    }) {
                      nodes {
                        id
                        usage
                        mediaType
                        colorMode
                        visualIndex
                      }
                    }
                }
            }
        }
    `);

    $effect(() => {
        if (player.currentTrackId) {
            trackQuery.fetch({ variables: { id: player.currentTrackId } });
        }
    });
    $effect(() => {
        console.log($trackQuery.data)
    })
    function handleEnded() {
        player.nextTrack();
    }
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
        {#if track?.trackMedia?.nodes[0]}
            <img class="cover" src="http://localhost:8000/media/{track?.trackMedia?.nodes[0].id}">
        {/if}
        </div>
        <div class="scrubber">
        <audio controls src="http://localhost:8000/track/{track.id}" autoplay bind:currentTime={player.currentTime} bind:paused={player.paused} onended={handleEnded}></audio>
        </div>
        <div class="queue">
            <h3>Queue</h3>
            {#if player.queue && player.queue.length > 0}
                <ol>
                    {#each player.queue as queueId, index}
                        <li style={index === player.queueIndex ? "font-weight: bold;" : ""}>Track ID: {queueId}</li>
                    {/each}
                </ol>
            {/if}
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

        display: grid;
        grid-auto-columns: 1fr;
        grid-template-columns: 1.2fr 0.8fr;
        grid-template-rows: 1fr auto;
        gap: 0px 0px;
        grid-template-areas:
          "track queue"
          "scrubber queue";
    }
    .track-info {
        grid-area: track;
    }
    .queue { grid-area: queue; }
    .scrubber { grid-area: scrubber; }
    .cover {max-width: 100%; width: 16em}
</style>
