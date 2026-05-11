import { env } from "$env/dynamic/public";
import { HoudiniClient } from "$houdini";

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

const client = new HoudiniClient({
    url: graph_endpoint.href,

    // uncomment this to configure the network call (for things like authentication)
    // for more information, please visit here: https://www.houdinigraphql.com/guides/authentication
    fetchParams({ session }) {
        return {
            headers: {
                ...(session?.token
                    ? { Authorization: `Bearer ${session.token}` }
                    : {}),
            },
        };
    },
});

export default client;
