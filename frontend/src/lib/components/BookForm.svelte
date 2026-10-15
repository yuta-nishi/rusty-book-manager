<script lang="ts">
	import { untrack } from 'svelte';
	import { ApiError } from '#lib/api/client.js';
	import type { CreateBookRequest } from '#lib/api/types.js';

	let {
		initial = { title: '', author: '', isbn: '', description: '' },
		submitLabel = '登録',
		cancelHref = '/books',
		onSubmit,
	}: {
		initial?: CreateBookRequest;
		submitLabel?: string;
		cancelHref?: string;
		onSubmit: (values: CreateBookRequest) => Promise<void>;
	} = $props();

	// The form seeds itself from `initial` once; edits are then local.
	let title = $state(untrack(() => initial.title));
	let author = $state(untrack(() => initial.author));
	let isbn = $state(untrack(() => initial.isbn));
	let description = $state(untrack(() => initial.description));
	let error = $state<string | null>(null);
	let submitting = $state(false);

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		error = null;
		submitting = true;
		try {
			await onSubmit({ title, author, isbn, description });
		} catch (cause) {
			error = cause instanceof ApiError ? cause.message : '保存に失敗しました。';
		} finally {
			submitting = false;
		}
	}
</script>

<div class="card">
	{#if error}
		<p class="error">{error}</p>
	{/if}
	<form onsubmit={submit}>
		<div class="field">
			<label for="title">タイトル</label>
			<input id="title" bind:value={title} required />
		</div>
		<div class="field-row">
			<div class="field">
				<label for="author">著者</label>
				<input id="author" bind:value={author} required />
			</div>
			<div class="field">
				<label for="isbn">ISBN</label>
				<input id="isbn" bind:value={isbn} required />
			</div>
		</div>
		<div class="field">
			<label for="description">説明</label>
			<textarea id="description" bind:value={description}></textarea>
		</div>
		<div class="actions">
			<button class="primary" type="submit" disabled={submitting}>
				{submitting ? '保存中…' : submitLabel}
			</button>
			<a class="button" href={cancelHref}>キャンセル</a>
		</div>
	</form>
</div>
