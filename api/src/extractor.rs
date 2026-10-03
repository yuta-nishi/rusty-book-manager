use axum::RequestPartsExt;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum_extra::TypedHeader;
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Bearer;
use domain::model::auth::AccessToken;
use domain::model::id::UserId;
use domain::model::role::Role;
use domain::model::user::User;
use registry::SharedAppRegistry;
use shared::error::AppError;

// Struct passed to handlers after request preprocessing.
pub struct AuthorizedUser {
    pub access_token: AccessToken,
    pub user: User,
}

impl AuthorizedUser {
    pub fn id(&self) -> UserId {
        self.user.id
    }
    pub fn is_admin(&self) -> bool {
        self.user.role == Role::Admin
    }
}

impl FromRequestParts<SharedAppRegistry> for AuthorizedUser {
    type Rejection = AppError;

    // Called when AuthorizedUser is added as a handler argument.
    async fn from_request_parts(
        parts: &mut Parts,
        registry: &SharedAppRegistry,
    ) -> Result<Self, Self::Rejection> {
        // Extract the access token from the HTTP header.
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AppError::UnauthorizedError)?;
        let access_token = AccessToken(bearer.token().to_string());

        // Extract the user ID associated with the access token.
        let user_id = registry
            .auth_repository()
            .fetch_user_id_from_token(&access_token)
            .await?
            .ok_or(AppError::UnauthenticatedError)?;

        // Look up the user record by user ID.
        let user = registry
            .user_repository()
            .find_current_user(user_id)
            .await?
            .ok_or(AppError::UnauthenticatedError)?;

        Ok(Self { access_token, user })
    }
}
