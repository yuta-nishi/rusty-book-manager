<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { ApiError } from '#lib/api/client.js';
	import { session } from '#lib/session.svelte.js';

	let email = $state('');
	let password = $state('');
	let error = $state<string | null>(null);
	let submitting = $state(false);

	const redirectTo = $derived(page.url.searchParams.get('redirectTo') ?? '/books');

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		error = null;
		submitting = true;
		try {
			await session.login(email, password);
			await goto(redirectTo, { invalidateAll: true });
		} catch (cause) {
			error = cause instanceof ApiError ? cause.message : 'ログインに失敗しました。';
		} finally {
			submitting = false;
		}
	}
</script>

<div class="login-page">
	<h1>ログイン</h1>
	<div class="card">
		{#if error}
			<p class="error">{error}</p>
		{/if}
		<form onsubmit={submit}>
			<div class="field">
				<label for="email">メールアドレス</label>
				<input
					id="email"
					type="email"
					bind:value={email}
					required
					autocomplete="username"
					placeholder="eleazar.fig@example.com"
				/>
			</div>
			<div class="field">
				<label for="password">パスワード</label>
				<input
					id="password"
					type="password"
					bind:value={password}
					required
					autocomplete="current-password"
				/>
			</div>
			<button class="primary" type="submit" disabled={submitting}>
				{submitting ? 'ログイン中…' : 'ログイン'}
			</button>
		</form>
	</div>
</div>
