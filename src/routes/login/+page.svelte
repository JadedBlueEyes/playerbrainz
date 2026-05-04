<script lang="ts">
    import { enhance } from "$app/forms";
    import { WhoamiQueryStore } from "$houdini";

    let { data, form } = $props();

    const whoamiStore = new WhoamiQueryStore();

    let whoamiError = $derived(
        $whoamiStore.errors?.map((e) => e.message).join(", "),
    );

    $effect(() => {
        if (data?.token) {
            whoamiStore.fetch({ policy: "NetworkOnly" });
        }
    });
</script>

<form method="POST" use:enhance>
    <h2>Login</h2>

    {#if form?.errorMessage}
        <p class="error">{form.errorMessage}</p>
    {/if}

    <div class="field">
        <label for="slug">Username (slug)</label>
        <input type="text" id="slug" name="slug" required />
    </div>

    <div class="field">
        <label for="password">Password</label>
        <input type="password" id="password" name="password" required />
    </div>

    <button type="submit">Sign In</button>
</form>

<div class="demo">
    {#if whoamiError}
        <p class="error" style="margin-top: 1rem;">Error: {whoamiError}</p>
    {/if}

    {#if $whoamiStore.data}
        <pre style="margin-top: 1rem; overflow-x: auto;">{JSON.stringify(
                $whoamiStore.data,
                null,
                2,
            )}</pre>
    {/if}
</div>

<style>
    form,
    .demo {
        display: flex;
        flex-direction: column;
        gap: 1rem;
        max-width: 24rem;
        margin: 2rem auto;
        padding: 1.5rem;
        background-color: #f0f0f0;
        border-radius: 8px;
    }

    .field {
        display: flex;
        flex-direction: column;
        gap: 0.25rem;
    }

    input {
        padding: 0.5rem;
        border: 1px solid #aaa;
        border-radius: 4px;
    }

    button {
        padding: 0.5rem;
        background-color: var(--primary-color);
        color: white;
        border: none;
        border-radius: 4px;
        cursor: pointer;
        font-weight: bold;
    }

    button:hover {
        background-color: oklch(from var(--primary-color) calc(l + 0.1) c h);
    }

    .error {
        color: var(--error-color);
        font-size: 0.85rem;
        margin: 0;
    }

    pre {
        font-size: 0.8rem;
        background: #f4f4f4;
        padding: 1rem;
        border-radius: 4px;
    }
</style>
