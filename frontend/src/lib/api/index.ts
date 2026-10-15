import { apiFetch, messageForStatus } from './client.js';
import type {
	AccessTokenResponse,
	BooksResponse,
	BookResponse,
	CheckoutsResponse,
	CreateBookRequest,
	CreateUserRequest,
	LoginRequest,
	UpdateBookRequest,
	UpdateUserPasswordRequest,
	UpdateUserRoleRequest,
	UserResponse,
	UsersResponse,
} from './types.js';

export { ApiError, messageForStatus } from './client.js';
export type * from './types.js';

/** Thin, typed wrapper over the backend's `/api/v1` endpoints. */
export const api = {
	// --- auth ---------------------------------------------------------------
	// Note: auth is mounted at `/auth`, not under `/api/v1`.
	login: (body: LoginRequest) =>
		apiFetch<AccessTokenResponse>('/auth/login', {
			method: 'POST',
			body,
			errorMessage: (status) =>
				status === 403
					? 'メールアドレスまたはパスワードが正しくありません。'
					: messageForStatus(status),
		}),
	logout: () => apiFetch<void>('/auth/logout', { method: 'POST' }),

	// --- users --------------------------------------------------------------
	me: () => apiFetch<UserResponse>('/api/v1/users/me'),
	myCheckouts: () => apiFetch<CheckoutsResponse>('/api/v1/users/me/checkouts'),
	changePassword: (body: UpdateUserPasswordRequest) =>
		apiFetch<void>('/api/v1/users/me/password', {
			method: 'PUT',
			body,
			errorMessage: (status) =>
				status === 403 ? '現在のパスワードが正しくありません。' : messageForStatus(status),
		}),
	listUsers: () => apiFetch<UsersResponse>('/api/v1/users'),
	registerUser: (body: CreateUserRequest) =>
		apiFetch<UserResponse>('/api/v1/users', {
			method: 'POST',
			body,
			errorMessage: (status) =>
				status === 403
					? 'ユーザーの登録は管理者のみが行えます。'
					: status === 400
						? '入力内容が不正です(メールアドレスの形式と必須項目を確認してください)。'
						: messageForStatus(status),
		}),
	deleteUser: (userId: string) => apiFetch<void>(`/api/v1/users/${userId}`, { method: 'DELETE' }),
	changeRole: (userId: string, body: UpdateUserRoleRequest) =>
		apiFetch<void>(`/api/v1/users/${userId}/role`, { method: 'PUT', body }),

	// --- books --------------------------------------------------------------
	listBooks: (options: { limit?: number; offset?: number } = {}) =>
		apiFetch<BooksResponse>('/api/v1/books', {
			query: { limit: options.limit, offset: options.offset },
		}),
	getBook: (bookId: string) => apiFetch<BookResponse>(`/api/v1/books/${bookId}`),
	createBook: (body: CreateBookRequest) =>
		apiFetch<void>('/api/v1/books', {
			method: 'POST',
			body,
			errorMessage: (status) =>
				status === 400
					? '入力内容が不正です(タイトル・著者・ISBN は必須です)。'
					: messageForStatus(status),
		}),
	updateBook: (bookId: string, body: UpdateBookRequest) =>
		apiFetch<void>(`/api/v1/books/${bookId}`, {
			method: 'PUT',
			body,
			errorMessage: (status) =>
				status === 404
					? '書籍が見つかりません。編集できるのは登録した本人だけです。'
					: status === 400
						? '入力内容が不正です(タイトル・著者・ISBN は必須です)。'
						: messageForStatus(status),
		}),
	deleteBook: (bookId: string) =>
		apiFetch<void>(`/api/v1/books/${bookId}`, {
			method: 'DELETE',
			errorMessage: (status) =>
				status === 404
					? '書籍が見つかりません。削除できるのは登録した本人だけです。'
					: messageForStatus(status),
		}),
	bookCheckoutHistory: (bookId: string) =>
		apiFetch<CheckoutsResponse>(`/api/v1/books/${bookId}/checkout-history`),

	// --- checkouts ----------------------------------------------------------
	listCheckouts: () => apiFetch<CheckoutsResponse>('/api/v1/books/checkouts'),
	checkoutBook: (bookId: string) =>
		apiFetch<void>(`/api/v1/books/${bookId}/checkouts`, {
			method: 'POST',
			errorMessage: (status) =>
				status === 422 ? 'この書籍はすでに貸出中です。' : messageForStatus(status),
		}),
	returnBook: (bookId: string, checkoutId: string) =>
		apiFetch<void>(`/api/v1/books/${bookId}/checkouts/${checkoutId}/returned`, {
			method: 'PUT',
			errorMessage: (status) =>
				status === 422
					? 'この書籍は返却できません(借りている本人のみ返却できます)。'
					: messageForStatus(status),
		}),
};
