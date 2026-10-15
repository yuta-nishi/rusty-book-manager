<script lang="ts">
	import { invalidateAll } from '$app/navigation';
	import { ApiError } from '#lib/api/client.js';
	import { api } from '#lib/api/index.js';
	import { formatDateTime } from '#lib/format.js';
	import { session } from '#lib/session.svelte.js';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	let error = $state<string | null>(null);
	let notice = $state<string | null>(null);
	let busy = $state(false);

	let currentPassword = $state('');
	let newPassword = $state('');

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

	async function changePassword(event: SubmitEvent) {
		event.preventDefault();
		await run(async () => {
			await api.changePassword({ currentPassword, newPassword });
			currentPassword = '';
			newPassword = '';
			notice = 'パスワードを変更しました。';
		});
	}

	async function returnBook(bookId: string, checkoutId: string) {
		await run(async () => {
			await api.returnBook(bookId, checkoutId);
			await session.refresh();
			await invalidateAll();
		});
	}
</script>

<h1>マイページ</h1>

{#if error}
	<p class="error">{error}</p>
{/if}
{#if notice}
	<p class="success">{notice}</p>
{/if}

<div class="card">
	<dl class="meta">
		<dt>名前</dt>
		<dd>{data.user.name}</dd>
		<dt>メールアドレス</dt>
		<dd>{data.user.email}</dd>
		<dt>ロール</dt>
		<dd><span class="badge">{data.user.role}</span></dd>
	</dl>
</div>

<h2>パスワードの変更</h2>
<div class="card">
	<form onsubmit={changePassword}>
		<div class="field-row">
			<div class="field">
				<label for="current-password">現在のパスワード</label>
				<input
					id="current-password"
					type="password"
					bind:value={currentPassword}
					required
					autocomplete="current-password"
				/>
			</div>
			<div class="field">
				<label for="new-password">新しいパスワード</label>
				<input
					id="new-password"
					type="password"
					bind:value={newPassword}
					required
					autocomplete="new-password"
				/>
			</div>
		</div>
		<button class="primary" type="submit" disabled={busy}>変更</button>
	</form>
</div>

<h2>貸出中の書籍</h2>
{#if data.checkouts.items.length === 0}
	<p class="empty">貸出中の書籍はありません。</p>
{:else}
	<table>
		<thead>
			<tr>
				<th>タイトル</th>
				<th>著者</th>
				<th>貸出日時</th>
				<th></th>
			</tr>
		</thead>
		<tbody>
			{#each data.checkouts.items as checkout (checkout.id)}
				<tr>
					<td><a href={`/books/${checkout.book.id}`}>{checkout.book.title}</a></td>
					<td>{checkout.book.author}</td>
					<td>{formatDateTime(checkout.checkedOutAt)}</td>
					<td>
						<button
							type="button"
							onclick={() => returnBook(checkout.book.id, checkout.id)}
							disabled={busy}
						>
							返却する
						</button>
					</td>
				</tr>
			{/each}
		</tbody>
	</table>
{/if}
