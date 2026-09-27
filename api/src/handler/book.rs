use crate::{
    extractor::AuthorizedUser,
    model::book::{
        BookResponse, BooksQuery, BooksResponse, CreateBookRequest, UpdateBookRequest,
        UpdateBookRequestWithIds,
    },
};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use garde::Validate;
use kernel::model::{book::event::DeleteBook, id::BookId};
use registry::AppRegistry;
use shared::error::{AppError, AppResult};

#[cfg_attr(
    debug_assertions,
    utoipa::path(post, path="/api/v1/books", tag = "books",
        summary = "Register a book",
        request_body = CreateBookRequest,
        responses(
            (status = 201, description = "The book was registered."),
            (status = 400, description = "The request was invalid."),
            (status = 401, description = "The user is not authenticated.")
        )
    )
)]
pub async fn register_book(
    user: AuthorizedUser,
    State(registry): State<AppRegistry>,
    Json(req): Json<CreateBookRequest>,
) -> AppResult<StatusCode> {
    req.validate()?;

    registry
        .book_repository()
        .create(req.into(), user.id())
        .await
        .map(|_| StatusCode::CREATED)
}

#[cfg_attr(
    debug_assertions,
    utoipa::path(get, path="/api/v1/books", tag = "books",
        summary = "List books",
        responses(
            (status = 200, description = "The book list was returned.", body = BooksResponse),
            (status = 400, description = "The query parameters were invalid."),
            (status = 401, description = "The user is not authenticated.")
        ),
        params(
            ("limit" = i64, Query, description = "Maximum number of books to return"),
            ("offset" = i64, Query, description = "Number of books to skip")
        )
    )
)]
pub async fn show_book_list(
    _user: AuthorizedUser,
    Query(query): Query<BooksQuery>,
    State(registry): State<AppRegistry>,
) -> AppResult<Json<BooksResponse>> {
    query.validate()?;

    registry
        .book_repository()
        .find_all(query.into())
        .await
        .map(BooksResponse::from)
        .map(Json)
}

#[cfg_attr(
    debug_assertions,
    utoipa::path(get, path="/api/v1/books/{book_id}", tag = "books",
        summary = "Get a book",
        responses(
            (status = 200, description = "The book was returned.", body = BookResponse),
            (status = 404, description = "The book was not found.")
        ),
        params(
            ("book_id" = Uuid, Path, description = "Book ID")
        )
    )
)]
pub async fn show_book(
    _user: AuthorizedUser,
    Path(book_id): Path<BookId>,
    State(registry): State<AppRegistry>,
) -> AppResult<Json<BookResponse>> {
    registry
        .book_repository()
        .find_by_id(book_id)
        .await
        .and_then(|bc| match bc {
            Some(bc) => Ok(Json(bc.into())),
            None => Err(AppError::EntityNotFound(
                "The specific book was not found".into(),
            )),
        })
}

#[cfg_attr(
    debug_assertions,
    utoipa::path(put, path="/api/v1/books/{book_id}", tag = "books",
        summary = "Update a book",
        request_body = UpdateBookRequest,
        responses(
            (status = 200, description = "The book was updated."),
            (status = 400, description = "The request was invalid."),
            (status = 404, description = "The book was not found.")
        ),
        params(
            ("book_id" = Uuid, Path, description = "Book ID")
        )
    )
)]
pub async fn update_book(
    user: AuthorizedUser,
    Path(book_id): Path<BookId>,
    State(registry): State<AppRegistry>,
    Json(req): Json<UpdateBookRequest>,
) -> AppResult<StatusCode> {
    req.validate()?;

    let update_book = UpdateBookRequestWithIds::new(book_id, user.id(), req);
    registry
        .book_repository()
        .update(update_book.into())
        .await
        .map(|_| StatusCode::OK)
}

#[cfg_attr(
    debug_assertions,
    utoipa::path(delete, path="/api/v1/books/{book_id}", tag = "books",
        summary = "Delete a book",
        responses(
            (status = 200, description = "The book was deleted."),
            (status = 404, description = "The book was not found.")
        ),
        params(
            ("book_id" = Uuid, Path, description = "Book ID")
        )
    )
)]
pub async fn delete_book(
    user: AuthorizedUser,
    Path(book_id): Path<BookId>,
    State(registry): State<AppRegistry>,
) -> AppResult<StatusCode> {
    let delete_book = DeleteBook {
        book_id,
        requested_user: user.id(),
    };
    registry
        .book_repository()
        .delete(delete_book)
        .await
        .map(|_| StatusCode::OK)
}
