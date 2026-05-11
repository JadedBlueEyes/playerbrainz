<script lang="ts">
    import { CreateUserStore } from "$houdini";
    import { goto } from "$app/navigation";

    const createUser = new CreateUserStore();

    let displayName = $state("");
    let slug = $state("");
    let password = $state("");
    let admin = false;

    async function handleSubmit(e: Event) {
        e.preventDefault();
        const result = await createUser.mutate({
            displayName: displayName !== "" ? displayName : null,
            slug,
            admin,
            password,
        });

        if (result.data?.createUser) {
            goto("/settings/users");
        }
    }
</script>

<div class="panel">
    <h2>New User</h2>

    <form onsubmit={handleSubmit} class="form">
        <label class="field">
            <span>Display Name</span>
            <input type="text" bind:value={displayName} />
        </label>

        <label class="field">
            <span>Slug</span>
            <input type="text" bind:value={slug} />
        </label>

        <label class="field">
            <span>Password</span>
            <input type="password" bind:value={password} />
        </label>

        <label class="field">
            <input type="checkbox" bind:checked={admin} />
            <span>Admin</span>
        </label>
        <div class="form-actions">
            <button type="submit" class="primary">Create User</button>
        </div>
    </form>
</div>

<style>
    @import "$lib/style/button.css";
    @import "$lib/style/form.css";
</style>
