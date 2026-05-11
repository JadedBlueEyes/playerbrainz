<script lang="ts">
    import { page } from "$app/state";
    import { UserStore, UpdateUserStore, DeleteUserStore } from "$houdini";
    import { goto } from "$app/navigation";
    import type { PageProps, RouteParams } from "./$types";

    import type { PageData } from "./$houdini";

    let { data, params }: { data: PageData; params: RouteParams } = $props();

    let { User: user } = $derived(data);

    const updateUser = new UpdateUserStore();
    const deleteUser = new DeleteUserStore();
    console.log($user?.data?.user);
    console.log($user);

    let id = $derived($user?.data?.user.id);
    let displayName = $state($user?.data?.user.displayName);
    let slug = $state($user?.data?.user.slug);
    let admin = $state($user?.data?.user.admin);

    async function handleUpdate(e: Event) {
        e.preventDefault();
        const result = await updateUser.mutate({
            id: parseInt(params.id),
            displayName: displayName != "" ? displayName : null,
            slug,
            admin,
        });

        if (result.data?.updateUser) {
            user.fetch();
        }
    }

    async function handleDelete() {
        const result = await deleteUser.mutate({
            id: parseInt(params.id),
        });

        if (result.data?.deleteUser) {
            goto("/settings/users");
        }
    }
</script>

<div class="panel">
    <h1>Edit User</h1>

    {#if $user.data?.user}
        <form onsubmit={handleUpdate} class="form">
            <label class="field">
                <span>Display Name</span>
                <input type="text" bind:value={displayName} />
            </label>

            <label class="field">
                <span>Slug</span>
                <input type="text" bind:value={slug} />
            </label>

            <label class="field">
                <input type="checkbox" bind:checked={admin} />
                <span>Admin</span>
            </label>

            <div class="form-actions">
                <button type="submit" class="primary">Update User</button>
            </div>
        </form>

        <button onclick={handleDelete} class=" danger">Delete User</button>
    {:else if $user.errors}
        {JSON.stringify($user.errors)}
    {/if}
</div>

<style>
    @import "$lib/style/button.css";
    @import "$lib/style/form.css";
</style>
