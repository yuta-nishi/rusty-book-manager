import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vite-plus/test';
import { ApiError } from '#lib/api/client.js';
import BookForm from '#lib/components/BookForm.svelte';

const initial = {
	title: 'Rust',
	author: '佐藤 大樹',
	isbn: '978-4-06-536957-9',
	description: '説明',
};

async function fillFields() {
	await fireEvent.input(screen.getByLabelText('タイトル'), { target: { value: 'Rust' } });
	await fireEvent.input(screen.getByLabelText('著者'), { target: { value: '佐藤 大樹' } });
	await fireEvent.input(screen.getByLabelText('ISBN'), { target: { value: '978-4-06-536957-9' } });
	await fireEvent.input(screen.getByLabelText('説明'), { target: { value: '説明' } });
}

describe('BookForm', () => {
	it('seeds the fields from the initial value', () => {
		render(BookForm, { props: { initial, onSubmit: vi.fn() } });

		expect(screen.getByLabelText('タイトル')).toHaveValue('Rust');
		expect(screen.getByLabelText('著者')).toHaveValue('佐藤 大樹');
		expect(screen.getByLabelText('ISBN')).toHaveValue('978-4-06-536957-9');
		expect(screen.getByLabelText('説明')).toHaveValue('説明');
	});

	it('starts empty and submits what the user typed', async () => {
		const onSubmit = vi.fn().mockResolvedValue(undefined);
		render(BookForm, { props: { onSubmit } });

		expect(screen.getByLabelText('タイトル')).toHaveValue('');
		await fillFields();
		await fireEvent.click(screen.getByRole('button', { name: '登録' }));

		expect(onSubmit).toHaveBeenCalledWith(initial);
	});

	it('shows the API error and lets the user retry', async () => {
		const onSubmit = vi
			.fn()
			.mockRejectedValue(
				new ApiError(400, '入力内容が不正です(タイトル・著者・ISBN は必須です)。'),
			);
		render(BookForm, { props: { onSubmit } });

		await fillFields();
		await fireEvent.click(screen.getByRole('button', { name: '登録' }));

		expect(
			await screen.findByText('入力内容が不正です(タイトル・著者・ISBN は必須です)。'),
		).toBeInTheDocument();
		expect(screen.getByRole('button', { name: '登録' })).toBeEnabled();
	});

	it('falls back to a generic message for unexpected failures', async () => {
		const onSubmit = vi.fn().mockRejectedValue(new Error('boom'));
		render(BookForm, { props: { onSubmit } });

		await fillFields();
		await fireEvent.click(screen.getByRole('button', { name: '登録' }));

		expect(await screen.findByText('保存に失敗しました。')).toBeInTheDocument();
	});
});
