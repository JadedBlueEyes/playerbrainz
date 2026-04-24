<script lang="ts">
    import { fragment, graphql, type TrackFragment } from '$houdini';
    import { getContext } from "svelte";

    let player: any = getContext('player');

    interface Props {
        track: any;
    }

    let { track }: Props = $props();

    let data = $derived(
        fragment(
            track,
            graphql(`
                fragment TrackFragment on Tracks {
                    artist
                    title
                    album
                    recordingId
                    id

                    trackMedia (filters:  {
                        usage:  {
                           eq: "FrontCover"
                        }

                    } orderBy: {
                       visualIndex: ASC
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
            `)
        )
    )

    function handleclick() {
        player().playTrack($data.id);
    }

    $effect(() => { console.log($data)})

    function handlecontextmenu(e: MouseEvent) {
        e.preventDefault();
        player().addTrackToQueue($data.id);
    }
</script>

<div class="track">
{#if $data?.trackMedia?.nodes[0]}
    <img class="cover" src="http://localhost:8000/media/{track?.trackMedia?.nodes[0].id}">
{/if}

<p data-id={$data.recordingId} onclick={handleclick} oncontextmenu={handlecontextmenu}><em>{$data.title}</em> by <em>{$data.artist}</em> from <em>{$data.album}</em></p>
</div>

<style>
    .track {
        display: inline-block;
        container-type: inline-size;
        /*max-width: 8em;*/
        flex: 0 0 auto;
        width: 8em;
    }
    .cover {
        max-width: 100%;
        display: inline-block;
    }
</style>
