use axum::{body::Body, http::Method, http::Request, http::StatusCode};
use rstest::rstest;
use tower::ServiceExt;

use crate::helper::{RequestBuilderExt, fixture, make_router, v1};
use registry::MockAppRegistry;

// The fixture's user has the User role, and no repository expectation is set:
// these endpoints have to answer 403 before they reach one.
async fn send(
    fixture: MockAppRegistry,
    method: Method,
    path: &str,
    body: Option<&str>,
) -> anyhow::Result<StatusCode> {
    let app = make_router(fixture);
    let builder = Request::builder().method(method).uri(v1(path)).bearer();
    let req = match body {
        Some(body) => builder
            .application_json()
            .body(Body::from(body.to_string()))?,
        None => builder.body(Body::empty())?,
    };

    Ok(app.oneshot(req).await?.status())
}

mod admin_only_endpoints {
    use super::*;

    #[rstest]
    #[case::register_user(
        Method::POST,
        "/users",
        Some(r#"{"name":"name","email":"user@example.com","password":"password"}"#)
    )]
    #[case::delete_user(
        Method::DELETE,
        "/users/00000000-0000-0000-0000-000000000000",
        None
    )]
    #[case::change_role(
        Method::PUT,
        "/users/00000000-0000-0000-0000-000000000000/role",
        Some(r#"{"role":"Admin"}"#)
    )]
    #[tokio::test]
    async fn reject_a_non_admin(
        fixture: MockAppRegistry,
        #[case] method: Method,
        #[case] path: &str,
        #[case] body: Option<&str>,
    ) -> anyhow::Result<()> {
        let status = send(fixture, method, path, body).await?;

        assert_eq!(status, StatusCode::FORBIDDEN);

        Ok(())
    }
}
