<script lang="ts">
	import { goto } from '$app/navigation';
	import { api } from '#lib/api/index.js';
	import BookForm from '#lib/components/BookForm.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<h1>書籍の編集</h1>

<BookForm
	initial={data.book}
	submitLabel="更新"
	cancelHref={`/books/${data.book.id}`}
	onSubmit={async (values) => {
		await api.updateBook(data.book.id, values);
		await goto(`/books/${data.book.id}`);
	}}
/>
