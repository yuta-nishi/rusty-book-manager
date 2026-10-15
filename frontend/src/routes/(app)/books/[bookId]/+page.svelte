<script lang="ts">
	import { goto, invalidateAll } from '$app/navigation';
	import { ApiError } from '#lib/api/client.js';
	import { api } from '#lib/api/index.js';
	import { formatDateTime, shortId } from '#lib/format.js';
	import { session } from '#lib/session.svelte.js';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	let error = $state<string | null>(null);
	let busy = $state(false);

	const isOwner = $derived(session.user?.id === data.book.owner.id);
	const isMyCheckout = $derived(data.book.checkout?.checkedOutBy.id === session.user?.id);

	async function run(action: () => Promise<void>) {
		error = null;
		busy = true;
		try {
			await action();
		} catch (cause) {
			error = cause instanceof ApiError ? cause.message : '処理に失敗しました。';
		} finally {
			busy = false;
		}
	}

	async function checkout() {
		await run(async () => {
			await api.checkoutBook(data.book.id);
			await invalidateAll();
		});
	}

	async function returnBook() {
		const checkout = data.book.checkout;
		if (checkout === null) return;
		await run(async () => {
			await api.returnBook(data.book.id, checkout.id);
			await invalidateAll();
		});
	}

	async function remove() {
		if (!confirm(`「${data.book.title}」を削除しますか?`)) return;
		await run(async () => {
			await api.deleteBook(data.book.id);
			await goto('/books');
		});
	}
</script>

<div class="page-header">
	<h1>{data.book.title}</h1>
	<div class="actions">
		{#if isOwner}
			<a class="button" href={`/books/${data.book.id}/edit`}>編集</a>
			<button class="danger" type="button" onclick={remove} disabled={busy}>削除</button>
		{/if}
	</div>
</div>

{#if error}
	<p class="error">{error}</p>
{/if}

<div class="card">
	<dl class="meta">
		<dt>著者</dt>
		<dd>{data.book.author}</dd>
		<dt>ISBN</dt>
		<dd>{data.book.isbn}</dd>
		<dt>所有者</dt>
		<dd>{data.book.owner.name}</dd>
		<dt>説明</dt>
		<dd>{data.book.description || '—'}</dd>
		<dt>状態</dt>
		<dd>
			{#if data.book.checkout}
				<span class="badge checked-out">
					{data.book.checkout.checkedOutBy.name} が {formatDateTime(
						data.book.checkout.checkedOutAt,
					)} に借りました
				</span>
			{:else}
				<span class="badge">貸出可</span>
			{/if}
		</dd>
	</dl>

	<div class="actions" style="margin-top: 1rem">
		{#if data.book.checkout === null}
			<button class="primary" type="button" onclick={checkout} disabled={busy}>借りる</button>
		{:else if isMyCheckout}
			<button class="primary" type="button" onclick={returnBook} disabled={busy}>返却する</button>
		{:else}
			<span class="muted">返却できるのは借りている本人だけです。</span>
		{/if}
	</div>
</div>

<h2>貸出履歴</h2>
{#if data.history.items.length === 0}
	<p class="empty">貸出履歴はありません。</p>
{:else}
	<table>
		<thead>
			<tr>
				<th>借りた人</th>
				<th>貸出日時</th>
				<th>返却日時</th>
			</tr>
		</thead>
		<tbody>
			{#each data.history.items as checkout (checkout.id)}
				<tr>
					<td
						>{checkout.checkedOutBy === session.user?.id
							? '自分'
							: shortId(checkout.checkedOutBy)}</td
					>
					<td>{formatDateTime(checkout.checkedOutAt)}</td>
					<td>
						{#if checkout.returnedAt}
							{formatDateTime(checkout.returnedAt)}
						{:else}
							<span class="badge checked-out">貸出中</span>
						{/if}
					</td>
				</tr>
			{/each}
		</tbody>
	</table>
{/if}

<p class="muted" style="margin-top: 1.5rem">書籍 ID: {data.book.id}</p>
