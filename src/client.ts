import { HoudiniClient } from "$houdini";

export default new HoudiniClient({
    url: "http://localhost:3030/graphql",

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
