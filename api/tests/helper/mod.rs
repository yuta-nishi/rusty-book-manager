use std::sync::Arc;

use api::routes::{auth, v1};
use axum::{Router, http::request::Builder};
use domain::{
    model::{auth::AccessToken, id::UserId, role::Role, user::User},
    repository::{auth::MockAuthRepository, user::MockUserRepository},
};
use registry::MockAppRegistry;
use rstest::fixture;

pub fn v1(endpoint: &str) -> String {
    format!("/api/v1{endpoint}")
}

pub fn make_router(registry: MockAppRegistry) -> Router {
    Router::new()
        .merge(v1::routes())
        .merge(auth::routes())
        .with_state(Arc::new(registry))
}

#[fixture]
pub fn fixture(#[default(Role::User)] role: Role) -> MockAppRegistry {
    let mut registry = MockAppRegistry::new();

    registry.expect_auth_repository().returning(|| {
        let mut mock = MockAuthRepository::new();
        mock.expect_fetch_user_id_from_token()
            .returning(|_| Ok(Some(UserId::new())));
        mock.expect_verify_user()
            .returning(|_, _| Ok(UserId::new()));
        mock.expect_create_token()
            .returning(|_| Ok(AccessToken("dummy".to_string())));
        Arc::new(mock)
    });

    registry.expect_user_repository().returning(move || {
        let mut mock = MockUserRepository::new();
        mock.expect_find_current_user().returning(move |id| {
            Ok(Some(User {
                id,
                name: "dummy-user".to_string(),
                email: "dummy@example.com".to_string(),
                role,
            }))
        });
        mock.expect_create().returning(|event| {
            Ok(User {
                id: UserId::new(),
                name: event.name,
                email: event.email,
                role: Role::User,
            })
        });
        mock.expect_delete().returning(|_| Ok(()));
        mock.expect_update_role().returning(|_| Ok(()));
        Arc::new(mock)
    });

    registry
}

pub trait RequestBuilderExt {
    fn bearer(self) -> Builder;
    fn application_json(self) -> Builder;
}

impl RequestBuilderExt for Builder {
    fn bearer(self) -> Builder {
        self.header("Authorization", "Bearer dummy")
    }

    fn application_json(self) -> Builder {
        self.header("Content-Type", "application/json")
    }
}
