import { fail } from '@sveltejs/kit';
import type { Actions } from './$types';

export const actions = {
	default: async ({ request, cookies }) => {
		const data = await request.formData();
		const slug = data.get('slug');
		const password = data.get('password');

		if (!slug || !password) {
			return fail(400, { errorMessage: 'Missing slug or password' });
		}

		try {
			const res = await fetch('http://localhost:3030/login', {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json'
				},
				body: JSON.stringify({ slug, password })
			});

			if (!res.ok) {
				return fail(res.status, { errorMessage: 'Invalid username or password' });
			}

			const result = await res.json();

			if (result.token) {
				cookies.set('token', result.token, {
					path: '/',
					httpOnly: true,
					secure: process.env.NODE_ENV === 'production',
					maxAge: 60 * 60 * 24 * 7 // 1 week
				});
			} else {
				return fail(500, { errorMessage: 'Login failed: no token received' });
			}
		} catch (error) {
			return fail(500, { errorMessage: 'Internal server error' });
		}

		return { success: true };
	}
} satisfies Actions;
