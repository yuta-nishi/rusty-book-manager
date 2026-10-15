<script lang="ts">
	import { goto } from '$app/navigation';
	import { api } from '#lib/api/index.js';
	import BookForm from '#lib/components/BookForm.svelte';
</script>

<h1>書籍の登録</h1>

<BookForm
	submitLabel="登録"
	onSubmit={async (values) => {
		await api.createBook(values);
		await goto('/books');
	}}
/>
