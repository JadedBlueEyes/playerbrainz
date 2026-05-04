import type { Handle } from "@sveltejs/kit";
import { setSession } from "$houdini";

export const handle: Handle = async ({ event, resolve }) => {
    const token = event.cookies.get("token");

    setSession(event, { token });

    return await resolve(event);
};
