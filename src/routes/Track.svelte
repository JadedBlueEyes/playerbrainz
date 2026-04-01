<script lang="ts">
    import { fragment, graphql, type TrackFragment } from '$houdini';
    // import {}} from "./$houdini";

    interface Props {
        track: TrackFragment
    }
    /** @type { import('undefined').Props } */
    let { track } = $props()

    let data = $derived(
        fragment(
            track,
            graphql(`
                fragment TrackFragment on Tracks {
                    artist
                    title
                    album
                    recordingId
                }
            `)
        )
    )
</script>

<!-- <img src="https://coverartarchive.org/release/{$data.recordingId}/front" loading="lazy"> -->
<p data-id={$data.recordingId}><em>{$data.title}</em> by <em>{$data.artist}</em> from <em>{$data.album}</em></p>
