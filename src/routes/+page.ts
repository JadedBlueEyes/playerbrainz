import { load_GetFlacTracks } from "$houdini";


export async function load(event: any) {
    return {
		...(await load_GetFlacTracks({
			event,
		}))
	};
}
