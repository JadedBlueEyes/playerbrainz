import { fail } from "@sveltejs/kit";
import type { Actions } from "./$types";
import { LoginStore } from "$houdini";

export const actions = {
    default: async (event) => {
        const { request, cookies } = event;

        const data = await request.formData();

        const slug = data.get("slug")?.toString();
        const password = data.get("password")?.toString();

        if (!slug || !password) {
            return fail(400, {
                errors: [{ message: "Missing slug or password" }],
            });
        }

        try {
            let login = new LoginStore();
            let res = await login.mutate({ slug, password }, { event });
            if (res.errors) {
                return fail(400, {
                    errors: res.errors,
                });
            }

            const result = res.data?.login;

            if (result?.token) {
                cookies.set("token", result.token, {
                    path: "/",
                    httpOnly: true,
                    secure: process.env.NODE_ENV === "production",
                    maxAge: 60 * 60 * 24 * 7, // 1 week
                });
            } else {
                return fail(500, {
                    errors: [{ message: "Login failed: no token received" }],
                });
            }
        } catch (error) {
            return fail(500, {
                errors: [{ message: "Internal server error" }],
            });
        }

        return { success: true };
    },
} satisfies Actions;
