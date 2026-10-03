use crate::database::{
    ConnectionPool,
    model::checkout::{CheckoutRow, CheckoutStateRow, ReturnedCheckoutRow},
};
use async_trait::async_trait;
use derive_new::new;
use kernel::model::checkout::{
    Checkout,
    event::{CreateCheckout, UpdateReturned},
};
use kernel::model::id::{BookId, CheckoutId, UserId};
use kernel::repository::checkout::CheckoutRepository;
use shared::error::{AppError, AppResult};

#[derive(new)]
pub struct CheckoutRepositoryImpl {
    db: ConnectionPool,
}

#[async_trait]
impl CheckoutRepository for CheckoutRepositoryImpl {
    async fn create(&self, event: CreateCheckout) -> AppResult<()> {
        let mut tx = self.db.begin().await?;
        self.set_transaction_serializable(&mut tx).await?;

        // Check that the book exists and is not already checked out.
        {
            let res = sqlx::query_as!(
                CheckoutStateRow,
                r#"
                    SELECT
                    b.book_id,
                    c.checkout_id AS "checkout_id?: CheckoutId",
                    NULL AS "user_id?: UserId"
                    FROM books AS b
                    LEFT OUTER JOIN checkouts AS c USING(book_id)
                    WHERE book_id = $1;
                "#,
                event.book_id as _
            )
            .fetch_optional(&mut *tx)
            .await
            .map_err(AppError::SpecificOperationError)?;

            match res {
                None => {
                    return Err(AppError::EntityNotFound(format!(
                        "book ({}) was not found",
                        event.book_id
                    )));
                }
                Some(CheckoutStateRow {
                    checkout_id: Some(_),
                    ..
                }) => {
                    return Err(AppError::UnprocessableEntity(format!(
                        "book ({}) is already checked out",
                        event.book_id
                    )));
                }
                _ => {}
            }
        }

        let checkout_id = CheckoutId::new();
        let res = sqlx::query!(
            r#"
                INSERT INTO checkouts
                (checkout_id, book_id, user_id, checked_out_at)
                VALUES ($1, $2, $3, $4)
                ;
            "#,
            checkout_id as _,
            event.book_id as _,
            event.checked_out_by as _,
            event.checked_out_at,
        )
        .execute(&mut *tx)
        .await
        .map_err(AppError::SpecificOperationError)?;

        if res.rows_affected() < 1 {
            return Err(AppError::NoRowsAffectedError(
                "No checkout record has been created".into(),
            ));
        }

        tx.commit().await.map_err(AppError::TransactionError)?;

        Ok(())
    }

    async fn update_returned(&self, event: UpdateReturned) -> AppResult<()> {
        let mut tx = self.db.begin().await?;
        self.set_transaction_serializable(&mut tx).await?;

        // Check that the book is checked out by the requesting user.
        {
            let res = sqlx::query_as!(
                CheckoutStateRow,
                r#"
                    SELECT
                    b.book_id,
                    c.checkout_id AS "checkout_id?: CheckoutId",
                    c.user_id AS "user_id?: UserId"
                    FROM books AS b
                    LEFT OUTER JOIN checkouts AS c USING(book_id)
                    WHERE book_id = $1;
                "#,
                event.book_id as _,
            )
            .fetch_optional(&mut *tx)
            .await
            .map_err(AppError::SpecificOperationError)?;

            match res {
                None => {
                    return Err(AppError::EntityNotFound(format!(
                        "book ({}) was not found",
                        event.book_id
                    )));
                }
                Some(CheckoutStateRow {
                    checkout_id: Some(c),
                    user_id: Some(u),
                    ..
                }) if (c, u) != (event.checkout_id, event.returned_by) => {
                    return Err(AppError::UnprocessableEntity(format!(
                        "checkout (id: {}, user: {}, book: {}) cannot be returned",
                        event.checkout_id, event.returned_by, event.book_id
                    )));
                }
                _ => {}
            }
        }

        // Move the record to returned_checkouts with returned_at set.
        let res = sqlx::query!(
            r#"
                INSERT INTO returned_checkouts
                (checkout_id, book_id, user_id, checked_out_at, returned_at)
                SELECT checkout_id, book_id, user_id, checked_out_at, $2
                FROM checkouts
                WHERE checkout_id = $1
                ;
            "#,
            event.checkout_id as _,
            event.returned_at,
        )
        .execute(&mut *tx)
        .await
        .map_err(AppError::SpecificOperationError)?;

        if res.rows_affected() < 1 {
            return Err(AppError::NoRowsAffectedError(
                "No returning record has been updated".into(),
            ));
        }

        let res = sqlx::query!(
            r#"
                DELETE FROM checkouts WHERE checkout_id = $1;
            "#,
            event.checkout_id as _,
        )
        .execute(&mut *tx)
        .await
        .map_err(AppError::SpecificOperationError)?;

        if res.rows_affected() < 1 {
            return Err(AppError::NoRowsAffectedError(
                "No checkout record has been deleted".into(),
            ));
        }

        tx.commit().await.map_err(AppError::TransactionError)?;

        Ok(())
    }

    async fn find_unreturned_all(&self) -> AppResult<Vec<Checkout>> {
        sqlx::query_as!(
            CheckoutRow,
            r#"
                SELECT
                c.checkout_id,
                c.book_id,
                c.user_id,
                c.checked_out_at,
                b.title,
                b.author,
                b.isbn
                FROM checkouts AS c
                INNER JOIN books AS b USING(book_id)
                ORDER BY c.checked_out_at ASC
                ;
            "#,
        )
        .fetch_all(self.db.inner_ref())
        .await
        .map(|rows| rows.into_iter().map(Checkout::from).collect())
        .map_err(AppError::SpecificOperationError)
    }

    async fn find_unreturned_by_user_id(
        &self,
        user_id: UserId,
    ) -> AppResult<Vec<Checkout>> {
        sqlx::query_as!(
            CheckoutRow,
            r#"
                SELECT
                c.checkout_id,
                c.book_id,
                c.user_id,
                c.checked_out_at,
                b.title,
                b.author,
                b.isbn
                FROM checkouts AS c
                INNER JOIN books AS b USING(book_id)
                WHERE c.user_id = $1
                ORDER BY c.checked_out_at ASC
                ;
            "#,
            user_id as _
        )
        .fetch_all(self.db.inner_ref())
        .await
        .map(|rows| rows.into_iter().map(Checkout::from).collect())
        .map_err(AppError::SpecificOperationError)
    }

    async fn find_history_by_book_id(&self, book_id: BookId) -> AppResult<Vec<Checkout>> {
        let checkout: Option<Checkout> = self.find_unreturned_by_book_id(book_id).await?;
        let mut checkout_histories: Vec<Checkout> = sqlx::query_as!(
            ReturnedCheckoutRow,
            r#"
                SELECT
                rc.checkout_id,
                rc.book_id,
                rc.user_id,
                rc.checked_out_at,
                rc.returned_at,
                b.title,
                b.author,
                b.isbn
                FROM returned_checkouts AS rc
                INNER JOIN books AS b USING(book_id)
                WHERE rc.book_id = $1
                ORDER BY rc.checked_out_at DESC
            "#,
            book_id as _
        )
        .fetch_all(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?
        .into_iter()
        .map(Checkout::from)
        .collect();

        // Put the currently checked-out record at the head of the history.
        if let Some(co) = checkout {
            checkout_histories.insert(0, co);
        }

        Ok(checkout_histories)
    }
}

impl CheckoutRepositoryImpl {
    // create / update_returned run at SERIALIZABLE to avoid double checkout.
    async fn set_transaction_serializable(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> AppResult<()> {
        sqlx::query!("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE")
            .execute(&mut **tx)
            .await
            .map_err(AppError::SpecificOperationError)?;
        Ok(())
    }

    async fn find_unreturned_by_book_id(
        &self,
        book_id: BookId,
    ) -> AppResult<Option<Checkout>> {
        let res = sqlx::query_as!(
            CheckoutRow,
            r#"
                SELECT
                c.checkout_id,
                c.book_id,
                c.user_id,
                c.checked_out_at,
                b.title,
                b.author,
                b.isbn
                FROM checkouts AS c
                INNER JOIN books AS b USING(book_id)
                WHERE c.book_id = $1
            "#,
            book_id as _,
        )
        .fetch_optional(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?
        .map(Checkout::from);

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::types::chrono::Utc;
    use std::str::FromStr;

    // IDs written by fixtures/checkout.sql.
    const BOOK_ID: &str = "9890736e-a4e4-461a-a77d-eac3517ef11b";
    const FIRST_USER_ID: &str = "9582f9de-0fd1-4892-b20c-70139a7eb95b";
    const SECOND_USER_ID: &str = "050afe56-c3da-4448-8e4d-6f44007d2ca5";

    #[sqlx::test(fixtures("common", "checkout"))]
    async fn checks_out_and_returns_a_book(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let repo = CheckoutRepositoryImpl::new(ConnectionPool::new(pool));
        let book_id = BookId::from_str(BOOK_ID)?;
        let first = UserId::from_str(FIRST_USER_ID)?;
        let second = UserId::from_str(SECOND_USER_ID)?;

        assert!(repo.find_unreturned_by_book_id(book_id).await?.is_none());

        // Checking out a book that does not exist is refused.
        let res = repo
            .create(CreateCheckout::new(BookId::new(), first, Utc::now()))
            .await;
        assert!(matches!(res, Err(AppError::EntityNotFound(_))));

        repo.create(CreateCheckout::new(book_id, first, Utc::now()))
            .await?;

        let checkout = repo.find_unreturned_by_book_id(book_id).await?.unwrap();
        assert_eq!(checkout.book.book_id, book_id);
        assert_eq!(checkout.checked_out_by, first);

        // The book is already out, so a second checkout is refused.
        let res = repo
            .create(CreateCheckout::new(book_id, second, Utc::now()))
            .await;
        assert!(matches!(res, Err(AppError::UnprocessableEntity(_))));

        // Returning it with another book or user is refused.
        let res = repo
            .update_returned(UpdateReturned::new(
                checkout.id,
                BookId::new(),
                first,
                Utc::now(),
            ))
            .await;
        assert!(matches!(res, Err(AppError::EntityNotFound(_))));

        let res = repo
            .update_returned(UpdateReturned::new(
                checkout.id,
                book_id,
                second,
                Utc::now(),
            ))
            .await;
        assert!(matches!(res, Err(AppError::UnprocessableEntity(_))));

        repo.update_returned(UpdateReturned::new(
            checkout.id,
            book_id,
            first,
            Utc::now(),
        ))
        .await?;

        assert!(repo.find_unreturned_by_book_id(book_id).await?.is_none());

        Ok(())
    }

    #[sqlx::test(fixtures("common", "checkout"))]
    async fn lists_checkouts(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let repo = CheckoutRepositoryImpl::new(ConnectionPool::new(pool));
        let book_id = BookId::from_str(BOOK_ID)?;
        let first = UserId::from_str(FIRST_USER_ID)?;
        let second = UserId::from_str(SECOND_USER_ID)?;

        repo.create(CreateCheckout::new(book_id, first, Utc::now()))
            .await?;
        let checkout = repo.find_unreturned_by_book_id(book_id).await?.unwrap();

        assert_eq!(repo.find_unreturned_all().await?.len(), 1);
        assert_eq!(repo.find_unreturned_by_user_id(first).await?.len(), 1);
        assert!(repo.find_unreturned_by_user_id(second).await?.is_empty());
        assert_eq!(repo.find_history_by_book_id(book_id).await?.len(), 1);

        repo.update_returned(UpdateReturned::new(
            checkout.id,
            book_id,
            first,
            Utc::now(),
        ))
        .await?;

        assert!(repo.find_unreturned_all().await?.is_empty());
        // The returned loan stays in the history.
        assert_eq!(repo.find_history_by_book_id(book_id).await?.len(), 1);

        repo.create(CreateCheckout::new(book_id, second, Utc::now()))
            .await?;
        let checkout = repo.find_unreturned_by_book_id(book_id).await?.unwrap();

        assert_eq!(repo.find_unreturned_all().await?.len(), 1);
        assert_eq!(repo.find_unreturned_by_user_id(second).await?.len(), 1);
        assert_eq!(repo.find_history_by_book_id(book_id).await?.len(), 2);

        repo.update_returned(UpdateReturned::new(
            checkout.id,
            book_id,
            second,
            Utc::now(),
        ))
        .await?;

        assert!(repo.find_unreturned_all().await?.is_empty());
        assert_eq!(repo.find_history_by_book_id(book_id).await?.len(), 2);

        Ok(())
    }
}
