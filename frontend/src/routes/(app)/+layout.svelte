<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { session } from '#lib/session.svelte.js';
	import type { LayoutProps } from './$types';

	let { children }: LayoutProps = $props();

	const pathname = $derived(page.url.pathname);

	async function logout() {
		await session.logout();
		await goto('/login');
	}
</script>

<header class="site-header">
	<a class="brand" href="/books">rusty-book-manager</a>
	<nav>
		<a href="/books" class:active={pathname.startsWith('/books')}>書籍</a>
		<a href="/checkouts" class:active={pathname.startsWith('/checkouts')}>貸出中</a>
		{#if session.isAdmin}
			<a href="/users" class:active={pathname.startsWith('/users')}>ユーザー</a>
		{/if}
		<a href="/me" class:active={pathname.startsWith('/me')}>マイページ</a>
	</nav>
	<div class="account">
		<span>{session.user?.name}</span>
		{#if session.isAdmin}<span class="badge">Admin</span>{/if}
		<button type="button" onclick={logout}>ログアウト</button>
	</div>
</header>

<main>
	{@render children()}
</main>
