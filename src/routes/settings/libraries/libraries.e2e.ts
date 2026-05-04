import { test, expect } from "@playwright/test";

test("login first, then manage filesystem libraries via UI", async ({
    page,
}) => {
    await page.goto("/login");

    const username = "admin";
    const password = "password";

    await page.getByLabel("Username (slug)").fill(username);
    await page.getByLabel("Password").fill(password);
    await page.getByRole("button", { name: "Sign In" }).click();

    await page.waitForLoadState("networkidle");

    await page.goto("/settings/libraries");

    await expect(
        page.getByRole("heading", {
            name: "Filesystem Libraries",
            exact: true,
        }),
    ).toBeVisible();
    await expect(
        page.getByRole("heading", { name: "Libraries", exact: true }),
    ).toBeVisible();
    await expect(
        page.getByRole("heading", { name: "Create", exact: true }),
    ).toBeVisible();
    await expect(
        page.getByRole("heading", { name: "Edit", exact: true }),
    ).toBeVisible();

    // Create a library

    const uniquePath = `/tmp/playerbrainz-e2e-fs-lib-${Date.now()}`;
    const initialName = "E2E Library";

    await page.getByLabel("Create path").fill(uniquePath);
    await page.getByLabel("Create display name").fill(initialName);
    await page.getByRole("button", { name: "Create", exact: true }).click();

    // Fail fast
    const errorBanner = page.locator("p.error");
    if (await errorBanner.isVisible()) {
        throw new Error(
            `UI error banner after Create: ${(await errorBanner.textContent()) ?? "(no text)"}`,
        );
    }

    const pageFetchError = page.getByText(/Error:\s*Failed to fetch/i);
    if (await pageFetchError.isVisible()) {
        throw new Error(
            `Libraries page error: ${(await pageFetchError.textContent()) ?? "(no text)"}`,
        );
    }
    // Expect library exists
    await expect(page.getByText(uniquePath, { exact: true })).toBeVisible();
    await expect(page.getByText(initialName, { exact: true })).toBeVisible();

    // Edit library
    await page.getByText(uniquePath, { exact: true }).click();

    const updatedName = "E2E Library (Updated)";
    await page.getByLabel("Edit display name").fill(updatedName);
    await page.getByRole("button", { name: "Save" }).click();

    if (await errorBanner.isVisible()) {
        throw new Error(
            `UI error banner after Save: ${(await errorBanner.textContent()) ?? "(no text)"}`,
        );
    }

    await expect(page.getByText(updatedName, { exact: true })).toBeVisible();

    // Deletion
    page.once("dialog", async (dialog) => {
        await dialog.accept();
    });

    const row = page.locator("li", { hasText: uniquePath });
    await row.getByRole("button", { name: "Delete" }).click();

    if (await errorBanner.isVisible()) {
        throw new Error(
            `UI error banner after Delete: ${(await errorBanner.textContent()) ?? "(no text)"}`,
        );
    }

    await expect(page.getByText(uniquePath, { exact: true })).not.toBeVisible();
});
