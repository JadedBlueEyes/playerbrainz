/// <references types="houdini-svelte">
let env = globalThis?.process?.env ?? (await import("$env/dynamic/public")).env;

let discovery_endpoint = new URL(
    "/.well-known/playerbrainz/client",
    env.PUBLIC_PLAYERBRAINZ_SERVER || "http://localhost:3030/",
);
let res = await fetch(discovery_endpoint);
let json = await res.json();

let graph_endpoint = new URL(
    json.graph_endpoint,
    env.PUBLIC_PLAYERBRAINZ_SERVER,
);

/** @type {import('houdini').ConfigFile} */
const config = {
    watchSchema: {
        url: graph_endpoint.href,
    },
    runtimeDir: ".houdini",
    plugins: {
        "houdini-svelte": {
            forceRunesMode: true,
        },
    },
};

export default config;
