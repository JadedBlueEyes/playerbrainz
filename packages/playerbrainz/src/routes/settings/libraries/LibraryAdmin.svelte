<script lang="ts">
    let {
        whoami,
        selectedId,
        busyCreate,
        busyUpdate,
        oncreate,
        onreset,
        onupdate,
    }: {
        whoami: { admin: boolean } | null;
        selectedId: number | null;
        busyCreate: boolean;
        busyUpdate: boolean;
        oncreate: (path: string, displayName: string) => Promise<void>;
        onupdate: (path: string, displayName: string) => Promise<void>;
        onreset: () => void;
    } = $props();

    let createPath = $state("");
    let createDisplayName = $state("");

    let editPath = $state("");
    let editDisplayName = $state("");

    export function setEditFields(path: string, displayName: string) {
        editPath = path;
        editDisplayName = displayName;
    }

    export function clearCreateFields() {
        createPath = "";
        createDisplayName = "";
    }
</script>

{#if whoami?.admin}
    <div class="panel">
        <h2>Create</h2>

        <div class="form">
            <label class="field" for="create-path">
                <span>Path</span>
                <input
                    id="create-path"
                    aria-label="Create path"
                    type="text"
                    bind:value={createPath}
                    autocomplete="off"
                    placeholder="/mnt/media/music"
                />
            </label>

            <label class="field" for="create-display-name">
                <span>Display name</span>
                <input
                    id="create-display-name"
                    aria-label="Create display name"
                    type="text"
                    bind:value={createDisplayName}
                    autocomplete="off"
                    placeholder="My Music"
                />
            </label>

            <div class="form-actions">
                <button
                    class="primary"
                    onclick={() => oncreate(createPath, createDisplayName)}
                    disabled={busyCreate}
                >
                    {#if busyCreate}Creating…{:else}Create{/if}
                </button>
            </div>
        </div>

        <hr />

        <h2>Edit</h2>

        {#if selectedId == null}
            <p class="muted">Select a library from the list to edit it.</p>
        {:else}
            <div class="form">
                <label class="field" for="edit-path">
                    <span>Path</span>
                    <input
                        id="edit-path"
                        aria-label="Edit path"
                        type="text"
                        bind:value={editPath}
                        autocomplete="off"
                    />
                </label>

                <label class="field" for="edit-display-name">
                    <span>Display name</span>
                    <input
                        id="edit-display-name"
                        aria-label="Edit display name"
                        type="text"
                        bind:value={editDisplayName}
                        autocomplete="off"
                    />
                </label>

                <div class="form-actions">
                    <button
                        class="secondary"
                        onclick={onreset}
                        disabled={busyUpdate}
                    >
                        Reset
                    </button>
                    <button
                        onclick={() => onupdate(editPath, editDisplayName)}
                        disabled={busyUpdate}
                    >
                        {#if busyUpdate}Saving…{:else}Save{/if}
                    </button>
                </div>
            </div>
        {/if}
    </div>
{:else}
    <div class="panel">
        <h2>Admin</h2>
        <p class="muted">
            You are not an admin. Ask an admin to manage filesystem libraries.
        </p>
    </div>
{/if}

<style>
    .panel {
        background: #f6f6f6;
        border: 1px solid #e6e6e6;
        border-radius: 10px;
        padding: 1rem;
    }

    h2 {
        margin: 0 0 0.75rem 0;
        font-size: 1.1rem;
    }

    .muted {
        color: #666;
        margin: 0.25rem 0 0;
    }

    hr {
        border: 0;
        border-top: 1px solid #e0e0e0;
        margin: 1.5rem 0;
    }
</style>
