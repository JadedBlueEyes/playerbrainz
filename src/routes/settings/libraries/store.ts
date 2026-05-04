import { goto } from "$app/navigation";
import {
    WhoamiQueryStore,
    FilesystemLibrariesQueryStore,
    CreateFilesystemLibraryMutationStore,
    UpdateFilesystemLibraryMutationStore,
    DeleteFilesystemLibraryMutationStore,
    type FilesystemLibrariesQuery$result,
    type WhoamiQuery$result,
} from "$houdini";

export type FilesystemLibraries =
    FilesystemLibrariesQuery$result["filesystemLibraries"];

export type Whoami = WhoamiQuery$result["whoami"];

function normalize(str: string) {
    return str.trim();
}

function toMessage(e: unknown) {
    if (e == null) return "Unknown error";
    if (typeof e === "string") return e;
    if (e instanceof Error) return e.message;
    return String(e);
}

function extractGqlMessage(e: unknown): string | null {
    const msg = toMessage(e);
    return msg ? msg : null;
}

function isNotLoggedIn(msg: string) {
    return msg.includes("Not logged in");
}

export const whoamiStore = new WhoamiQueryStore();
export const librariesStore = new FilesystemLibrariesQueryStore();
export const createStore = new CreateFilesystemLibraryMutationStore();
export const updateStore = new UpdateFilesystemLibraryMutationStore();
export const deleteStore = new DeleteFilesystemLibraryMutationStore();

export async function checkSession(): Promise<{
    whoami: Whoami | null;
    error: string | null;
}> {
    try {
        const result = await whoamiStore.fetch({ policy: "NetworkOnly" });
        const whoami = (result as any)?.data?.whoami ?? null;

        if (!whoami) {
            await goto("/login", { replaceState: false });
            return { whoami: null, error: "Not logged in" };
        }
        return { whoami, error: null };
    } catch (e: unknown) {
        const msg = extractGqlMessage(e);
        if (msg && isNotLoggedIn(msg)) {
            await goto("/login", { replaceState: false });
            return { whoami: null, error: "Not logged in" };
        }
        return { whoami: null, error: msg };
    }
}

export async function refreshLibraries(): Promise<{
    libraries: FilesystemLibraries;
    error: string | null;
}> {
    try {
        const result = await librariesStore.fetch({ policy: "NetworkOnly" });
        const libraries = ((result as any)?.data?.filesystemLibraries ??
            []) as FilesystemLibraries;
        return { libraries, error: null };
    } catch (e: unknown) {
        const msg = extractGqlMessage(e);
        if (msg && isNotLoggedIn(msg)) {
            await goto("/login", { replaceState: true });
            return { libraries: [], error: "Not logged in" };
        }
        return { libraries: [], error: msg };
    }
}

export async function createLibrary(
    path: string,
    displayName: string,
): Promise<{ error: string | null }> {
    const normalPath = normalize(path);
    const normalDisplayName = normalize(displayName);

    if (!normalPath) {
        return { error: "Path is required." };
    }

    try {
        const input: Record<string, unknown> = { path: normalPath };
        if (normalDisplayName.length > 0) input.displayName = normalDisplayName;

        await createStore.mutate({
            input,
        } as any);

        return { error: null };
    } catch (e: unknown) {
        const msg = extractGqlMessage(e);
        if (msg && isNotLoggedIn(msg)) {
            await goto("/login", { replaceState: true });
            return { error: "Not logged in" };
        }
        return { error: msg };
    }
}

export async function updateLibrary(
    library: FilesystemLibraries[0],
    path: string,
    displayName: string,
): Promise<{ updated?: FilesystemLibraries[0]; error: string | null }> {
    const nextPath = normalize(path);
    const nextDisplayName = normalize(displayName);

    if (!nextPath) {
        return { error: "Path is required." };
    }

    try {
        const input: Record<string, unknown> = { id: library.id };

        if (nextPath !== library.path) input.path = nextPath;
        if ((library.displayName ?? "") !== nextDisplayName) {
            input.displayName =
                nextDisplayName.length > 0 ? nextDisplayName : null;
        }

        const result = await updateStore.mutate({
            input,
        } as any);

        const updated = (result as any)?.data?.updateFilesystemLibrary as
            | FilesystemLibraries[0]
            | undefined;

        return { updated, error: null };
    } catch (e: unknown) {
        const msg = extractGqlMessage(e);
        if (msg && isNotLoggedIn(msg)) {
            await goto("/login", { replaceState: true });
            return { error: "Not logged in" };
        }
        return { error: msg };
    }
}

export async function deleteLibrary(
    id: number,
): Promise<{ error: string | null }> {
    try {
        await deleteStore.mutate({
            input: { id },
        } as any);
        return { error: null };
    } catch (e: unknown) {
        const msg = extractGqlMessage(e);
        if (msg && isNotLoggedIn(msg)) {
            await goto("/login", { replaceState: true });
            return { error: "Not logged in" };
        }
        return { error: msg };
    }
}
