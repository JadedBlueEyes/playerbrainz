import { test, expect } from "@playwright/test";

test.describe("Login flow", () => {
    test("can submit the login form and successfully fetch whoami data", async ({
        page,
    }) => {
        // Navigate to the login page
        await page.goto("/login");

        // Check if the heading is present
        await expect(
            page.getByRole("heading", { name: "Login", exact: true }),
        ).toBeVisible();

        // Fill in the login form credentials
        await page.fill('input[name="slug"]', "admin");
        await page.fill('input[name="password"]', "password");

        // Submit the form
        await page.click('button[type="submit"]');

        // Wait for the result pre block to appear
        const resultPre = page.locator(".demo pre");
        await expect(resultPre).toBeVisible({ timeout: 5000 });

        // Assert the exact structure and predictable values of the whoami output
        const outputText = await resultPre.textContent();
        expect(outputText).not.toBeNull();

        const parsedOutput = JSON.parse(outputText as string);

        expect(parsedOutput).toMatchObject({
            slug: "admin",
            admin: true,
        });

        // Assert that the timestamp and ID fields are present
        expect(typeof parsedOutput.id).toBe("number");
        expect(typeof parsedOutput.createdAt).toBe("string");
        expect(typeof parsedOutput.updatedAt).toBe("string");
    });
});
