use crate::{extractor::AuthorizedUser, model::checkout::CheckoutsResponse};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use composition::SharedAppRegistry;
use domain::model::{
    checkout::event::{CreateCheckout, UpdateReturned},
    id::{BookId, CheckoutId},
};
use shared::error::AppResult;

#[cfg_attr(
    debug_assertions,
    utoipa::path(post, path="/api/v1/books/{book_id}/checkouts", tag = "books",
        summary = "Check out a book",
        responses(
            (status = 201, description = "The book was checked out."),
            (status = 404, description = "The book was not found."),
            (status = 422, description = "The book is already checked out.")
        ),
        params(
            ("book_id" = Uuid, Path, description = "Book ID")
        )
    )
)]
pub async fn checkout_book(
    user: AuthorizedUser,
    Path(book_id): Path<BookId>,
    State(registry): State<SharedAppRegistry>,
) -> AppResult<StatusCode> {
    let create_checkout_history =
        CreateCheckout::new(book_id, user.id(), chrono::Utc::now());

    registry
        .checkout_repository()
        .create(create_checkout_history)
        .await
        .map(|_| StatusCode::CREATED)
}

#[cfg_attr(
    debug_assertions,
    utoipa::path(put, path="/api/v1/books/{book_id}/checkouts/{checkout_id}/returned", tag = "books",
        summary = "Return a book",
        responses(
            (status = 200, description = "The book was returned."),
            (status = 404, description = "The book was not found."),
            (status = 422, description = "The checkout cannot be returned.")
        ),
        params(
            ("book_id" = Uuid, Path, description = "Book ID"),
            ("checkout_id" = Uuid, Path, description = "Checkout ID")
        )
    )
)]
pub async fn return_book(
    user: AuthorizedUser,
    Path((book_id, checkout_id)): Path<(BookId, CheckoutId)>,
    State(registry): State<SharedAppRegistry>,
) -> AppResult<StatusCode> {
    let update_returned =
        UpdateReturned::new(checkout_id, book_id, user.id(), chrono::Utc::now());

    registry
        .checkout_repository()
        .update_returned(update_returned)
        .await
        .map(|_| StatusCode::OK)
}

#[cfg_attr(
    debug_assertions,
    utoipa::path(get, path="/api/v1/books/checkouts", tag = "books",
        summary = "List checked out books",
        responses(
            (status = 200, description = "The checked out list was returned.", body = CheckoutsResponse)
        )
    )
)]
pub async fn show_checked_out_list(
    _user: AuthorizedUser,
    State(registry): State<SharedAppRegistry>,
) -> AppResult<Json<CheckoutsResponse>> {
    registry
        .checkout_repository()
        .find_unreturned_all()
        .await
        .map(CheckoutsResponse::from)
        .map(Json)
}

#[cfg_attr(
    debug_assertions,
    utoipa::path(get, path="/api/v1/books/{book_id}/checkout-history", tag = "books",
        summary = "Get the checkout history of a book",
        responses(
            (status = 200, description = "The checkout history was returned.", body = CheckoutsResponse)
        ),
        params(
            ("book_id" = Uuid, Path, description = "Book ID")
        )
    )
)]
pub async fn checkout_history(
    _user: AuthorizedUser,
    Path(book_id): Path<BookId>,
    State(registry): State<SharedAppRegistry>,
) -> AppResult<Json<CheckoutsResponse>> {
    registry
        .checkout_repository()
        .find_history_by_book_id(book_id)
        .await
        .map(CheckoutsResponse::from)
        .map(Json)
}
