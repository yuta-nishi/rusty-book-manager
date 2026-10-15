// Types mirroring the JSON shapes exposed by the Rust backend under `/api/v1`.
//
// All IDs are UUIDs serialized as hyphen-less lowercase hex strings
// (see `domain::model::id`), so they are treated as opaque `string`s here.
// Timestamps are RFC 3339 / ISO 8601 strings (chrono `DateTime<Utc>`).

export type UserId = string;
export type BookId = string;
export type CheckoutId = string;

export type RoleName = 'Admin' | 'User';

export interface LoginRequest {
	email: string;
	password: string;
}

export interface AccessTokenResponse {
	userId: UserId;
	accessToken: string;
}

export interface UserResponse {
	id: UserId;
	name: string;
	email: string;
	role: RoleName;
}

export interface UsersResponse {
	items: UserResponse[];
}

export interface CreateUserRequest {
	name: string;
	email: string;
	password: string;
}

export interface UpdateUserRoleRequest {
	role: RoleName;
}

export interface UpdateUserPasswordRequest {
	currentPassword: string;
	newPassword: string;
}

export interface BookOwnerResponse {
	id: UserId;
	name: string;
}

export interface CheckoutUserResponse {
	id: UserId;
	name: string;
}

/** The active (unreturned) checkout of a book, embedded in `BookResponse`. */
export interface BookCheckoutResponse {
	id: CheckoutId;
	checkedOutBy: CheckoutUserResponse;
	checkedOutAt: string;
}

export interface BookResponse {
	id: BookId;
	title: string;
	author: string;
	isbn: string;
	description: string;
	owner: BookOwnerResponse;
	checkout: BookCheckoutResponse | null;
}

export interface BooksResponse {
	total: number;
	limit: number;
	offset: number;
	items: BookResponse[];
}

export interface CreateBookRequest {
	title: string;
	author: string;
	isbn: string;
	description: string;
}

export type UpdateBookRequest = CreateBookRequest;

export interface CheckoutBookResponse {
	id: BookId;
	title: string;
	author: string;
	isbn: string;
}

export interface CheckoutResponse {
	id: CheckoutId;
	checkedOutBy: UserId;
	checkedOutAt: string;
	returnedAt: string | null;
	book: CheckoutBookResponse;
}

export interface CheckoutsResponse {
	items: CheckoutResponse[];
}
