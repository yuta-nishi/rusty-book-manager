use std::sync::Arc;

use api::routes::{auth, v1};
use axum::{Router, http::request::Builder, response::Response};
use kernel::{
    model::{auth::AccessToken, id::UserId, role::Role, user::User},
    repository::{auth::MockAuthRepository, user::MockUserRepository},
};
use registry::MockAppRegistry;
use rstest::fixture;
use serde::de::DeserializeOwned;

pub fn v1(endpoint: &str) -> String {
    format!("/api/v1{endpoint}")
}

pub fn make_router(registry: MockAppRegistry) -> Router {
    Router::new()
        .merge(v1::routes())
        .merge(auth::routes())
        .with_state(Arc::new(registry))
}

// Every handler behind AuthorizedUser needs the token check and the user lookup
// to answer, so they are stubbed here rather than in each test.
#[fixture]
pub fn fixture() -> MockAppRegistry {
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

    registry.expect_user_repository().returning(|| {
        let mut mock = MockUserRepository::new();
        mock.expect_find_current_user().returning(|id| {
            Ok(Some(User {
                id,
                name: "dummy-user".to_string(),
                email: "dummy@example.com".to_string(),
                role: Role::User,
            }))
        });
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

pub async fn deserialize_json<T: DeserializeOwned>(
    response: Response,
) -> anyhow::Result<T> {
    use tokio_stream::StreamExt;

    let mut bytes = Vec::new();
    let mut stream = response.into_body().into_data_stream();
    while let Ok(Some(chunk)) = stream.try_next().await {
        bytes.extend_from_slice(&chunk[..]);
    }

    Ok(serde_json::from_slice(&bytes)?)
}
