use crate::{
    extractor::AuthorizedUser,
    model::user::{
        CreateUserRequest, UpdateUserPasswordRequest,
        UpdateUserPasswordRequestWithUserId, UpdateUserRoleRequest,
        UpdateUserRoleRequestWithUserId, UserResponse, UsersResponse,
    },
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use garde::Validate;
use kernel::model::{id::UserId, user::event::DeleteUser};
use registry::AppRegistry;
use shared::error::{AppError, AppResult};

#[cfg_attr(
    debug_assertions,
    utoipa::path(post, path="/api/v1/users", tag = "users",
        summary = "Register a user",
        request_body = CreateUserRequest,
        responses(
            (status = 200, description = "The user was registered.", body = UserResponse),
            (status = 400, description = "The request was invalid."),
            (status = 403, description = "The user is not allowed to perform this operation.")
        )
    )
)]
pub async fn register_user(
    user: AuthorizedUser,
    State(registry): State<AppRegistry>,
    Json(req): Json<CreateUserRequest>,
) -> AppResult<Json<UserResponse>> {
    //AuthorizedUser の権限が Admin のときのみ実行可能とする
    if !user.is_admin() {
        return Err(AppError::ForbiddenOperation);
    }
    req.validate()?;

    let registered_user = registry.user_repository().create(req.into()).await?;

    Ok(Json(registered_user.into()))
}

#[cfg_attr(
    debug_assertions,
    utoipa::path(get, path="/api/v1/users", tag = "users",
        summary = "List users",
        responses(
            (status = 200, description = "The user list was returned.", body = UsersResponse),
            (status = 500, description = "An internal server error occurred.")
        )
    )
)]
pub async fn list_users(
    _user: AuthorizedUser,
    State(registry): State<AppRegistry>,
) -> AppResult<Json<UsersResponse>> {
    let items = registry
        .user_repository()
        .find_all()
        .await?
        .into_iter()
        .map(UserResponse::from)
        .collect();

    Ok(Json(UsersResponse { items }))
}

#[cfg_attr(
    debug_assertions,
    utoipa::path(delete, path="/api/v1/users/{user_id}", tag = "users",
        summary = "Delete a user",
        responses(
            (status = 200, description = "The user was deleted."),
            (status = 403, description = "The user is not allowed to perform this operation."),
            (status = 404, description = "The user was not found.")
        ),
        params(
            ("user_id" = Uuid, Path, description = "User ID")
        )
    )
)]
pub async fn delete_user(
    user: AuthorizedUser,
    Path(user_id): Path<UserId>,
    State(registry): State<AppRegistry>,
) -> AppResult<StatusCode> {
    //AuthorizedUser の権限が Admin のときのみ実行可能とする
    if !user.is_admin() {
        return Err(AppError::ForbiddenOperation);
    }

    registry
        .user_repository()
        .delete(DeleteUser { user_id })
        .await?;

    Ok(StatusCode::OK)
}

#[cfg_attr(
    debug_assertions,
    utoipa::path(put, path="/api/v1/users/{user_id}/role", tag = "users",
        summary = "Change a user's role",
        request_body = UpdateUserRoleRequest,
        responses(
            (status = 200, description = "The role was changed."),
            (status = 403, description = "The user is not allowed to perform this operation."),
            (status = 404, description = "The user was not found.")
        ),
        params(
            ("user_id" = Uuid, Path, description = "User ID")
        )
    )
)]
pub async fn change_role(
    user: AuthorizedUser,
    Path(user_id): Path<UserId>,
    State(registry): State<AppRegistry>,
    Json(req): Json<UpdateUserRoleRequest>,
) -> AppResult<StatusCode> {
    //AuthorizedUser の権限が Admin のときのみ実行可能とする
    if !user.is_admin() {
        return Err(AppError::ForbiddenOperation);
    }

    registry
        .user_repository()
        .update_role(UpdateUserRoleRequestWithUserId::new(user_id, req).into())
        .await?;

    Ok(StatusCode::OK)
}

#[cfg_attr(
    debug_assertions,
    utoipa::path(get, path="/api/v1/users/me", tag = "users",
        summary = "Get the current user",
        responses(
            (status = 200, description = "The current user was returned.", body = UserResponse)
        )
    )
)]
pub async fn get_current_user(user: AuthorizedUser) -> Json<UserResponse> {
    Json(UserResponse::from(user.user))
}

#[cfg_attr(
    debug_assertions,
    utoipa::path(put, path="/api/v1/users/me/password", tag = "users",
        summary = "Change the current user's password",
        request_body = UpdateUserPasswordRequest,
        responses(
            (status = 200, description = "The password was changed."),
            (status = 400, description = "The request was invalid."),
            (status = 500, description = "An internal server error occurred.")
        )
    )
)]
pub async fn change_password(
    user: AuthorizedUser,
    State(registry): State<AppRegistry>,
    Json(req): Json<UpdateUserPasswordRequest>,
) -> AppResult<StatusCode> {
    req.validate()?;

    registry
        .user_repository()
        .update_password(UpdateUserPasswordRequestWithUserId::new(user.id(), req).into())
        .await?;

    Ok(StatusCode::OK)
}
