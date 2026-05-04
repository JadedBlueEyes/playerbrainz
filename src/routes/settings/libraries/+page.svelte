<script lang="ts">
    import {
        checkSession,
        refreshLibraries,
        createLibrary,
        updateLibrary,
        deleteLibrary,
        type Whoami,
        type FilesystemLibraries,
        whoamiStore,
        librariesStore,
        createStore,
        updateStore,
        deleteStore,
    } from "./store";
    import LibraryList from "./LibraryList.svelte";
    import LibraryAdmin from "./LibraryAdmin.svelte";

    let errorMessage = $state<string | null>(null);

    let whoami = $state<Whoami | null>(null);

    let libraries = $state<FilesystemLibraries>([]);
    let selectedId = $state<number | null>(null);

    let busyDelete = $state<number | null>(null);

    let adminPanel: LibraryAdmin | null = $state(null);

    function selectedLibrary(): FilesystemLibraries[0] | null {
        if (selectedId == null) return null;
        return libraries.find((l) => l.id === selectedId) ?? null;
    }

    function pickLibrary(id: number) {
        selectedId = id;
        const lib = selectedLibrary();
        if (lib) {
            adminPanel?.setEditFields(lib.path, lib.displayName ?? "");
        }
        errorMessage = null;
    }

    async function handleRefresh() {
        errorMessage = null;

        const { libraries: newLibraries, error } = await refreshLibraries();
        if (error) {
            errorMessage = error;
            libraries = [];
        } else {
            libraries = newLibraries;
        }

        if (selectedId != null && !libraries.some((l) => l.id === selectedId)) {
            selectedId = null;
        }
    }

    $effect(() => {
        async function initialLoad() {
            const { whoami: session, error } = await checkSession();

            if (error) {
                errorMessage = error;
                whoami = null;
            } else {
                whoami = session;
                if (whoami) {
                    handleRefresh();
                }
            }
        }
        initialLoad();
    });

    async function onCreate(path: string, displayName: string) {
        if (!whoami?.admin) {
            errorMessage = "Admin only";
            return;
        }

        errorMessage = null;

        const { error } = await createLibrary(path, displayName);
        if (error) {
            errorMessage = error;
        } else {
            adminPanel?.clearCreateFields();
            await handleRefresh();
        }
    }

    async function onUpdate(path: string, displayName: string) {
        if (!whoami?.admin) {
            errorMessage = "Admin only";
            return;
        }

        const lib = selectedLibrary();
        if (!lib) {
            errorMessage = "Select a library to update.";
            return;
        }

        errorMessage = null;

        const { updated, error } = await updateLibrary(lib, path, displayName);
        if (error) {
            errorMessage = error;
        } else {
            await handleRefresh();
            if (updated?.id != null) {
                pickLibrary(updated.id);
            }
        }
    }

    async function onDelete(id: number) {
        if (!whoami?.admin) {
            errorMessage = "Admin only";
            return;
        }

        const lib = libraries.find((l) => l.id === id);
        if (!lib) return;

        const ok = confirm(
            `Delete library #${lib.id}?\n\n${lib.displayName ?? "(no display name)"}\n${lib.path}`,
        );
        if (!ok) return;

        busyDelete = id;
        errorMessage = null;

        const { error } = await deleteLibrary(id);
        if (error) {
            errorMessage = error;
        } else {
            if (selectedId === id) {
                selectedId = null;
            }
            await handleRefresh();
        }

        busyDelete = null;
    }
</script>

<div class="page">
    <header class="header">
        <div class="title">
            <h1>Filesystem Libraries</h1>
        </div>

        <div class="actions">
            <button
                class="secondary"
                onclick={handleRefresh}
                disabled={$librariesStore.fetching}
            >
                {#if $librariesStore.fetching}Refreshing…{:else}Refresh{/if}
            </button>
        </div>
    </header>

    {#if errorMessage}
        <p class="error">Error: {errorMessage}</p>
    {/if}

    {#if $whoamiStore.fetching}
        <p class="muted">Checking session…</p>
    {:else}
        <section class="grid">
            {#if $librariesStore.fetching && libraries.length === 0}
                <p class="muted">Loading libraries…</p>
            {:else}
                <LibraryList
                    {libraries}
                    {selectedId}
                    {busyDelete}
                    onselect={pickLibrary}
                    ondelete={onDelete}
                />
            {/if}

            <LibraryAdmin
                bind:this={adminPanel}
                {whoami}
                {selectedId}
                busyCreate={$createStore.fetching}
                busyUpdate={$updateStore.fetching}
                oncreate={onCreate}
                onupdate={onUpdate}
                onreset={() => pickLibrary(selectedId!)}
            />
        </section>
    {/if}
</div>

<style>
    @import "$lib/style/button.css";
    @import "$lib/style/form.css";
    .page {
        max-width: 1100px;
        margin: 2rem auto;
        padding: 0 1rem;
    }

    .header {
        display: flex;
        align-items: flex-start;
        justify-content: space-between;
        gap: 1rem;
        margin-bottom: 1rem;
    }

    .title h1 {
        margin: 0;
    }

    .actions {
        display: flex;
        gap: 0.5rem;
    }

    .error {
        color: var(--error-color);
        background: color-mix(in oklab, var(--error-color) 10%, white);
        border: 1px solid color-mix(in oklab, var(--error-color) 25%, white);
        padding: 0.75rem;
        border-radius: 8px;
        margin: 0.75rem 0 1rem;
        white-space: pre-wrap;
    }

    .grid {
        display: grid;
        grid-template-columns: 1.2fr 0.8fr;
        gap: 1rem;
    }

    @media (max-width: 900px) {
        .grid {
            grid-template-columns: 1fr;
        }
    }

    .muted {
        color: #666;
        margin: 0.25rem 0 0;
    }
</style>
