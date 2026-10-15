<script lang="ts">
	import { goto } from '$app/navigation';
	import { invalidateAll } from '$app/navigation';
	import { ApiError } from '#lib/api/client.js';
	import { api } from '#lib/api/index.js';
	import { session } from '#lib/session.svelte.js';
	import type { RoleName, UserResponse } from '#lib/api/types.js';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	let error = $state<string | null>(null);
	let notice = $state<string | null>(null);
	let busy = $state(false);

	let name = $state('');
	let email = $state('');
	let password = $state('');

	async function run(action: () => Promise<void>) {
		error = null;
		notice = null;
		busy = true;
		try {
			await action();
		} catch (cause) {
			error = cause instanceof ApiError ? cause.message : '処理に失敗しました。';
		} finally {
			busy = false;
		}
	}

	async function register(event: SubmitEvent) {
		event.preventDefault();
		await run(async () => {
			const created = await api.registerUser({ name, email, password });
			name = '';
			email = '';
			password = '';
			notice = `${created.name} を登録しました。`;
			await invalidateAll();
		});
	}

	async function changeRole(userId: string, role: RoleName) {
		await run(async () => {
			await api.changeRole(userId, { role });
			if (userId === session.user?.id) await session.refresh();
			await invalidateAll();
		});
	}

	async function remove(user: UserResponse) {
		if (!confirm(`${user.name} を削除しますか?`)) return;
		await run(async () => {
			await api.deleteUser(user.id);
			if (user.id === session.user?.id) {
				session.clear();
				await goto('/login');
				return;
			}
			await invalidateAll();
		});
	}
</script>

<h1>ユーザー管理</h1>

{#if error}
	<p class="error">{error}</p>
{/if}
{#if notice}
	<p class="success">{notice}</p>
{/if}

<h2>ユーザーの登録</h2>
<div class="card">
	<form onsubmit={register}>
		<div class="field-row">
			<div class="field">
				<label for="name">名前</label>
				<input id="name" bind:value={name} required />
			</div>
			<div class="field">
				<label for="email">メールアドレス</label>
				<input id="email" type="email" bind:value={email} required />
			</div>
			<div class="field">
				<label for="password">パスワード</label>
				<input id="password" type="password" bind:value={password} required />
			</div>
		</div>
		<button class="primary" type="submit" disabled={busy}>登録</button>
	</form>
</div>

<h2>ユーザー一覧</h2>
<table>
	<thead>
		<tr>
			<th>名前</th>
			<th>メールアドレス</th>
			<th>ロール</th>
			<th></th>
		</tr>
	</thead>
	<tbody>
		{#each data.users.items as user (user.id)}
			<tr>
				<td>
					{user.name}
					{#if user.id === session.user?.id}<span class="badge">自分</span>{/if}
				</td>
				<td>{user.email}</td>
				<td>
					<select
						value={user.role}
						onchange={(event) => changeRole(user.id, event.currentTarget.value as RoleName)}
						disabled={busy}
					>
						<option value="Admin">Admin</option>
						<option value="User">User</option>
					</select>
				</td>
				<td>
					<button class="danger" type="button" onclick={() => remove(user)} disabled={busy}>
						削除
					</button>
				</td>
			</tr>
		{/each}
	</tbody>
</table>
