import { load_User } from "$houdini";

import type { PageLoad } from "./$houdini";

export const load: PageLoad = async (event) => {
    return {
        ...(await load_User({
            event,
            variables: { id: parseInt(event.params.id) },
        })),
    };
};
