<script lang="ts">
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const { limit, offset, total } = $derived(data.books);
	const hasPrev = $derived(offset > 0);
	const hasNext = $derived(offset + limit < total);
	const rangeStart = $derived(total === 0 ? 0 : offset + 1);
	const rangeEnd = $derived(Math.min(offset + limit, total));
</script>

<div class="page-header">
	<h1>書籍一覧</h1>
	<a class="button primary" href="/books/new">書籍を登録</a>
</div>

{#if data.books.items.length === 0}
	<p class="empty">書籍がまだ登録されていません。</p>
{:else}
	<table>
		<thead>
			<tr>
				<th>タイトル</th>
				<th>著者</th>
				<th>ISBN</th>
				<th>所有者</th>
				<th>状態</th>
			</tr>
		</thead>
		<tbody>
			{#each data.books.items as book (book.id)}
				<tr>
					<td><a href={`/books/${book.id}`}>{book.title}</a></td>
					<td>{book.author}</td>
					<td>{book.isbn}</td>
					<td>{book.owner.name}</td>
					<td>
						{#if book.checkout}
							<span class="badge checked-out">貸出中: {book.checkout.checkedOutBy.name}</span>
						{:else}
							<span class="badge">貸出可</span>
						{/if}
					</td>
				</tr>
			{/each}
		</tbody>
	</table>

	<div class="pager">
		{#if hasPrev}
			<a class="button" href={`/books?limit=${limit}&offset=${Math.max(offset - limit, 0)}`}>前へ</a
			>
		{/if}
		<span class="muted">{rangeStart}–{rangeEnd} / {total} 件</span>
		{#if hasNext}
			<a class="button" href={`/books?limit=${limit}&offset=${offset + limit}`}>次へ</a>
		{/if}
	</div>
{/if}
