import { load_Users } from "$houdini";

import type { PageLoad } from "./$houdini";

export const load: PageLoad = async (event) => {
    return {
        ...(await load_Users({ event })),
    };
};
