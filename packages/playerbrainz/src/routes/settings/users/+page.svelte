<script lang="ts">
    import { UsersStore } from "$houdini";

    import type { PageData } from "./$houdini";

    let { data }: { data: PageData } = $props();

    let { Users: users } = $derived(data);
    // svelte-ignore state_referenced_locally
</script>

<div class="panel">
    <h1>Users</h1>

    <ul class="user-list">
        {#each $users.data?.users ?? [] as user}
            <li>
                <a href="/settings/users/{user.id}" class="user-item">
                    {#if user.displayName}
                        <span class="user-display-name">{user.displayName}</span
                        >
                    {/if}
                    <span class="user-slug">{user.slug}</span>
                    {#if user.admin}
                        <div class="admin-badge">Admin</div>
                    {/if}
                </a>
            </li>
        {/each}
    </ul>

    <a href="/settings/users/new" class="button primary"> New User </a>
</div>

<style>
    @import "$lib/style/button.css";
    @import "$lib/style/form.css";
</style>
