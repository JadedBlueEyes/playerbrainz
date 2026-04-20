<script lang="ts">
    import { fragment, graphql, type TrackFragment } from '$houdini';
    import { getContext } from "svelte";

    let player: any = getContext('player');

    interface Props {
        track: TrackFragment;
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
                }
            `)
        )
    )

    function handleclick() {
        player().currentTrackId = $data.id;
    }
</script>

<!-- <img src="https://coverartarchive.org/release/{$data.recordingId}/front" loading="lazy"> -->
<p data-id={$data.recordingId} onclick={handleclick}><em>{$data.title}</em> by <em>{$data.artist}</em> from <em>{$data.album}</em></p>
