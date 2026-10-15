<script lang="ts">
	import { invalidateAll } from '$app/navigation';
	import { ApiError } from '#lib/api/client.js';
	import { api } from '#lib/api/index.js';
	import { formatDateTime, shortId } from '#lib/format.js';
	import { session } from '#lib/session.svelte.js';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	let error = $state<string | null>(null);
	let busy = $state(false);

	async function returnBook(bookId: string, checkoutId: string) {
		error = null;
		busy = true;
		try {
			await api.returnBook(bookId, checkoutId);
			await invalidateAll();
		} catch (cause) {
			error = cause instanceof ApiError ? cause.message : '返却に失敗しました。';
		} finally {
			busy = false;
		}
	}
</script>

<h1>貸出中の書籍</h1>

{#if error}
	<p class="error">{error}</p>
{/if}

{#if data.checkouts.items.length === 0}
	<p class="empty">貸出中の書籍はありません。</p>
{:else}
	<table>
		<thead>
			<tr>
				<th>タイトル</th>
				<th>借りている人</th>
				<th>貸出日時</th>
				<th></th>
			</tr>
		</thead>
		<tbody>
			{#each data.checkouts.items as checkout (checkout.id)}
				<tr>
					<td><a href={`/books/${checkout.book.id}`}>{checkout.book.title}</a></td>
					<td
						>{checkout.checkedOutBy === session.user?.id
							? '自分'
							: shortId(checkout.checkedOutBy)}</td
					>
					<td>{formatDateTime(checkout.checkedOutAt)}</td>
					<td>
						{#if checkout.checkedOutBy === session.user?.id}
							<button
								type="button"
								onclick={() => returnBook(checkout.book.id, checkout.id)}
								disabled={busy}
							>
								返却する
							</button>
						{/if}
					</td>
				</tr>
			{/each}
		</tbody>
	</table>
{/if}
