mod helper;

use axum::{
    body::Body, http::Method, http::Request, http::StatusCode, response::Response,
};
use kernel::model::role::Role;
use rstest::rstest;
use tower::ServiceExt;

use helper::{RequestBuilderExt, fixture, make_router, v1};
use registry::MockAppRegistry;

async fn send(
    fixture: MockAppRegistry,
    method: Method,
    path: &str,
    body: Option<&str>,
) -> anyhow::Result<Response> {
    let app = make_router(fixture);
    let builder = Request::builder().method(method).uri(v1(path)).bearer();
    let req = match body {
        Some(body) => builder
            .application_json()
            .body(Body::from(body.to_string()))?,
        None => builder.body(Body::empty())?,
    };

    Ok(app.oneshot(req).await?)
}

mod register_user {
    use super::*;

    const PATH: &str = "/users";
    const BODY: &str =
        r#"{"name":"name","email":"user@example.com","password":"password"}"#;

    #[rstest]
    #[tokio::test]
    async fn creates_a_user(
        #[with(Role::Admin)] fixture: MockAppRegistry,
    ) -> anyhow::Result<()> {
        let response = send(fixture, Method::POST, PATH, Some(BODY)).await?;

        assert_eq!(response.status(), StatusCode::OK);

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn rejects_a_non_admin(fixture: MockAppRegistry) -> anyhow::Result<()> {
        let response = send(fixture, Method::POST, PATH, Some(BODY)).await?;

        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        Ok(())
    }
}

mod delete_user {
    use super::*;

    const PATH: &str = "/users/00000000-0000-0000-0000-000000000000";

    #[rstest]
    #[tokio::test]
    async fn deletes_a_user(
        #[with(Role::Admin)] fixture: MockAppRegistry,
    ) -> anyhow::Result<()> {
        let response = send(fixture, Method::DELETE, PATH, None).await?;

        assert_eq!(response.status(), StatusCode::OK);

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn rejects_a_non_admin(fixture: MockAppRegistry) -> anyhow::Result<()> {
        let response = send(fixture, Method::DELETE, PATH, None).await?;

        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        Ok(())
    }
}

mod change_role {
    use super::*;

    const PATH: &str = "/users/00000000-0000-0000-0000-000000000000/role";
    const BODY: &str = r#"{"role":"Admin"}"#;

    #[rstest]
    #[tokio::test]
    async fn changes_the_role(
        #[with(Role::Admin)] fixture: MockAppRegistry,
    ) -> anyhow::Result<()> {
        let response = send(fixture, Method::PUT, PATH, Some(BODY)).await?;

        assert_eq!(response.status(), StatusCode::OK);

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn rejects_a_non_admin(fixture: MockAppRegistry) -> anyhow::Result<()> {
        let response = send(fixture, Method::PUT, PATH, Some(BODY)).await?;

        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        Ok(())
    }
}
