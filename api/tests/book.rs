use std::sync::Arc;

mod helper;

use axum::{body::Body, http::Request, http::StatusCode, response::Response};
use rstest::rstest;
use serde::de::DeserializeOwned;
use tower::ServiceExt;

use api::model::book::BookResponse;
use composition::MockAppRegistry;
use domain::{
    model::{
        book::{Book, Checkout},
        id::{BookId, CheckoutId, UserId},
        user::{BookOwner, CheckoutUser},
    },
    repository::book::MockBookRepository,
};
use helper::{RequestBuilderExt, fixture, make_router, v1};

fn book(book_id: BookId, checkout: Option<CheckoutId>) -> Book {
    Book {
        id: book_id,
        title: "Rust Web Application Development".to_string(),
        author: "Author".to_string(),
        isbn: "978-4-297-13656-6".to_string(),
        description: "Sample application for the book".to_string(),
        owner: BookOwner {
            id: UserId::new(),
            name: "Owner".to_string(),
        },
        checkout: checkout.map(|checkout_id| Checkout {
            checkout_id,
            checked_out_by: CheckoutUser {
                id: UserId::new(),
                name: "Borrower".to_string(),
            },
            checked_out_at: "2026-01-01T00:00:00Z".parse().unwrap(),
        }),
    }
}

async fn post_book(
    fixture: MockAppRegistry,
    payload: &str,
) -> anyhow::Result<StatusCode> {
    let app = make_router(fixture);
    let req = Request::post(v1("/books"))
        .application_json()
        .bearer()
        .body(Body::from(payload.to_string()))?;

    Ok(app.oneshot(req).await?.status())
}

async fn get_book(fixture: MockAppRegistry, book_id: BookId) -> anyhow::Result<Response> {
    let app = make_router(fixture);
    let req = Request::get(v1(&format!("/books/{book_id}")))
        .bearer()
        .body(Body::empty())?;

    Ok(app.oneshot(req).await?)
}

async fn deserialize_json<T: DeserializeOwned>(response: Response) -> anyhow::Result<T> {
    use tokio_stream::StreamExt;

    let mut bytes = Vec::new();
    let mut stream = response.into_body().into_data_stream();
    while let Ok(Some(chunk)) = stream.try_next().await {
        bytes.extend_from_slice(&chunk[..]);
    }

    Ok(serde_json::from_slice(&bytes)?)
}

mod register_book {
    use super::*;

    #[rstest]
    #[tokio::test]
    async fn creates_a_book_with_a_valid_body(
        mut fixture: MockAppRegistry,
    ) -> anyhow::Result<()> {
        fixture.expect_book_repository().returning(|| {
            let mut mock = MockBookRepository::new();
            mock.expect_create().returning(|_, _| Ok(()));
            Arc::new(mock)
        });

        let status = post_book(
            fixture,
            r#"{"title":"Book","author":"Author","isbn":"isbn","description":""}"#,
        )
        .await?;

        assert_eq!(status, StatusCode::CREATED);

        Ok(())
    }

    #[rstest]
    #[case::blank_title(
        r#"{"title":"","author":"Author","isbn":"isbn","description":""}"#
    )]
    #[case::blank_author(
        r#"{"title":"Book","author":"","isbn":"isbn","description":""}"#
    )]
    #[case::missing_body("")]
    #[tokio::test]
    async fn rejects_invalid_requests(
        fixture: MockAppRegistry,
        #[case] payload: &str,
    ) -> anyhow::Result<()> {
        let status = post_book(fixture, payload).await?;

        assert_eq!(status, StatusCode::BAD_REQUEST);

        Ok(())
    }
}

mod show_book {
    use super::*;

    #[rstest]
    #[tokio::test]
    async fn reports_no_checkout_when_the_book_is_available(
        mut fixture: MockAppRegistry,
    ) -> anyhow::Result<()> {
        let book_id = BookId::new();
        fixture.expect_book_repository().returning(move || {
            let mut mock = MockBookRepository::new();
            mock.expect_find_by_id()
                .returning(move |_| Ok(Some(book(book_id, None))));
            Arc::new(mock)
        });

        let response = get_book(fixture, book_id).await?;
        assert_eq!(response.status(), StatusCode::OK);

        let body: BookResponse = deserialize_json(response).await?;
        assert!(body.checkout.is_none());

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn reports_the_borrower_when_the_book_is_checked_out(
        mut fixture: MockAppRegistry,
    ) -> anyhow::Result<()> {
        let book_id = BookId::new();
        let checkout_id = CheckoutId::new();
        fixture.expect_book_repository().returning(move || {
            let mut mock = MockBookRepository::new();
            mock.expect_find_by_id()
                .returning(move |_| Ok(Some(book(book_id, Some(checkout_id)))));
            Arc::new(mock)
        });

        let response = get_book(fixture, book_id).await?;
        assert_eq!(response.status(), StatusCode::OK);

        let body: BookResponse = deserialize_json(response).await?;
        let checkout = body.checkout.expect("the checkout should be reported");
        assert_eq!(checkout.id, checkout_id);
        assert_eq!(checkout.checked_out_by.name, "Borrower");

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn returns_404_when_the_book_does_not_exist(
        mut fixture: MockAppRegistry,
    ) -> anyhow::Result<()> {
        fixture.expect_book_repository().returning(|| {
            let mut mock = MockBookRepository::new();
            mock.expect_find_by_id().returning(|_| Ok(None));
            Arc::new(mock)
        });

        let response = get_book(fixture, BookId::new()).await?;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        Ok(())
    }
}
