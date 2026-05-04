<script lang="ts">
  import type { FilesystemLibraries } from "./store";

  let {
    libraries,
    selectedId,
    busyDelete,
    onselect,
    ondelete
  }: {
    libraries: FilesystemLibraries;
    selectedId: number | null;
    busyDelete: number | null;
    onselect: (id: number) => void
    ondelete: (id: number) => void
  } = $props();

</script>

<div class="panel">
  <h2>Libraries</h2>

  {#if libraries.length === 0}
    <p class="muted">No filesystem libraries yet.</p>
  {:else}
    <ul class="list" aria-label="Filesystem libraries list">
      {#each libraries as lib (lib.id)}
        <li class:selected={lib.id === selectedId}>
          <div class="row">
            <button
              class="row-select"
              type="button"
              onclick={() => onselect(lib.id)}
            >
              <div class="row-main">
                <div class="row-title">
                  <span class="name"
                    >{lib.displayName ?? "(no display name)"}</span
                  >
                  <span class="id">#{lib.id}</span>
                </div>
                <div class="path">{lib.path}</div>
                <div class="meta">
                  <span
                    >Created: {new Date(lib.createdAt).toLocaleString()}</span
                  >
                  <span
                    >Updated: {new Date(lib.updatedAt).toLocaleString()}</span
                  >
                </div>
              </div>
            </button>

            <div class="row-actions">
              <button
                class="danger"
                type="button"
                onclick={() => ondelete(lib.id)}
                disabled={busyDelete != null && busyDelete !== lib.id}
              >
                {#if busyDelete === lib.id}Deleting…{:else}Delete{/if}
              </button>
            </div>
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</div>

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

  .list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .list li {
    border-radius: 12px;
    border: 1px solid #e4e4e4;
    background: white;

    overflow: hidden;
  }

  .list li.selected {
    border-color: color-mix(in oklab, var(--primary-color) 55%, white);
    box-shadow: 0 0 0 1px color-mix(in oklab, var(--primary-color) 35%, white);
  }

  .row {
    display: flex;
    align-items: stretch;
    justify-content: space-between;
    text-align: left;
  }

  .row:hover {
    background: #fafafa;
  }

  .row-select {
    flex: 1;
    text-align: left;
    background: transparent;
    border: 0;
    color: inherit;
    cursor: pointer;
  }

  .row-main {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .row-title {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 0.5rem;
  }

  .name {
    font-weight: 650;
  }

  .id {
    color: #666;
    font-size: 0.9rem;
  }

  .path {
    font-family:
      ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono",
      "Courier New", monospace;
    font-size: 0.9rem;
    color: #222;

    word-break: break-all;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .meta {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;

    font-size: 0.75rem;
  }

  .row-actions {
    display: flex;
    align-items: center;
    padding: 0.55rem 0.75rem;
  }
</style>
