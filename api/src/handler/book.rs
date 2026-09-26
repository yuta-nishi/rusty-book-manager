use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use kernel::model::id::BookId;
use registry::AppRegistry;
use shared::error::{AppError, AppResult};

use crate::model::book::{BookResponse, CreateBookRequest};

#[cfg_attr(
    debug_assertions,
    utoipa::path(post, path="/api/v1/books", tag = "books",
        summary = "Register a book",
        request_body = CreateBookRequest,
        responses(
            (status = 201, description = "The book was registered."),
            (status = 400, description = "The request was invalid.")
        )
    )
)]
pub async fn register_book(
    State(registry): State<AppRegistry>,
    Json(req): Json<CreateBookRequest>,
) -> AppResult<StatusCode> {
    registry
        .book_repository()
        .create(req.into())
        .await
        .map(|_| StatusCode::CREATED)
}

#[cfg_attr(
    debug_assertions,
    utoipa::path(get, path="/api/v1/books", tag = "books",
        summary = "List books",
        responses(
            (status = 200, description = "The book list was returned.", body = [BookResponse]),
            (status = 400, description = "The query parameters were invalid.")
        )
    )
)]
pub async fn show_book_list(
    State(registry): State<AppRegistry>,
) -> AppResult<Json<Vec<BookResponse>>> {
    registry
        .book_repository()
        .find_all()
        .await
        .map(|v| v.into_iter().map(BookResponse::from).collect::<Vec<_>>())
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
