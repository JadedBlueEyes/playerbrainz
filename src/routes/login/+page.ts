import { WhoamiQueryStore, load_WhoamiQuery } from "$houdini";

import type { PageLoad } from "./$houdini";

export const load: PageLoad = async (event) => {
    return {
        ...(await load_WhoamiQuery({ event })),
    };
};
