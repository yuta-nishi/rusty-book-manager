use crate::{handler, model};

#[derive(utoipa::OpenApi)]
#[openapi(
    info(
        title = "rusty-book-manager",
        description = "Sample application for the book 'Rust Web Application Development'.",
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "books", description = "Book management"),
        (name = "users", description = "User management"),
        (name = "auth", description = "Authentication"),
    ),
    paths(
        handler::health::health_check,
        handler::health::health_check_db,
        handler::book::show_book_list,
        handler::book::show_book,
        handler::book::register_book,
        handler::book::update_book,
        handler::book::delete_book,
        handler::checkout::checkout_book,
        handler::checkout::return_book,
        handler::checkout::show_checked_out_list,
        handler::checkout::checkout_history,
        handler::user::get_current_user,
        handler::user::list_users,
        handler::user::register_user,
        handler::user::delete_user,
        handler::user::change_role,
        handler::user::change_password,
        handler::user::get_checkouts,
        handler::auth::login,
        handler::auth::logout,
    ),
    components(schemas(
        model::book::CreateBookRequest,
        model::book::UpdateBookRequest,
        model::book::BookResponse,
        model::book::BooksResponse,
        model::checkout::CheckoutsResponse,
        model::checkout::CheckoutResponse,
        model::checkout::CheckoutBookResponse,
        model::user::BookOwnerResponse,
        model::user::RoleName,
        model::user::UserResponse,
        model::user::UsersResponse,
        model::user::CreateUserRequest,
        model::user::UpdateUserRoleRequest,
        model::user::UpdateUserPasswordRequest,
        model::auth::LoginRequest,
        model::auth::AccessTokenResponse,
        kernel::model::id::BookId,
        kernel::model::id::UserId,
    ))
)]
pub struct ApiDoc;
