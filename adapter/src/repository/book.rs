use async_trait::async_trait;
use derive_new::new;

use crate::database::ConnectionPool;
use crate::database::model::book::{BookCheckoutRow, BookRow, PaginatedBookRow};
use kernel::model::{
    book::{
        Book, BooksOptions, Checkout,
        event::{CreateBook, DeleteBook, UpdateBook},
    },
    id::{BookId, UserId},
    list::PaginatedList,
};
use kernel::repository::book::BookRepository;
use shared::error::{AppError, AppResult};
use std::collections::HashMap;

#[derive(new)]
pub struct BookRepositoryImpl {
    db: ConnectionPool,
}

#[async_trait]
impl BookRepository for BookRepositoryImpl {
    async fn create(&self, event: CreateBook, user_id: UserId) -> AppResult<()> {
        sqlx::query!(
            r#"
                INSERT INTO books (title, author, isbn, description, user_id)
                VALUES ($1, $2, $3, $4, $5)
            "#,
            event.title,
            event.author,
            event.isbn,
            event.description,
            user_id as _
        )
        .execute(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;

        Ok(())
    }

    // A page of books needs three queries: the page of IDs (which carries the
    // total), the rows of those books, and the checkouts attached to them.
    async fn find_all(&self, options: BooksOptions) -> AppResult<PaginatedList<Book>> {
        let BooksOptions { limit, offset } = options;

        let id_rows: Vec<PaginatedBookRow> = sqlx::query_as!(
            PaginatedBookRow,
            r#"
                SELECT
                COUNT(*) OVER() AS "total!",
                b.book_id AS id
                FROM books AS b
                ORDER BY b.created_at DESC
                LIMIT $1
                OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;

        // COUNT(*) OVER() is computed before LIMIT/OFFSET, so an empty page has
        // no row carrying the total and the count falls back to 0.
        let total = id_rows.first().map(|r| r.total).unwrap_or_default();
        let book_ids = id_rows.into_iter().map(|r| r.id).collect::<Vec<BookId>>();

        let book_rows: Vec<BookRow> = sqlx::query_as!(
            BookRow,
            r#"
                SELECT
                    b.book_id AS book_id,
                    b.title AS title,
                    b.author AS author,
                    b.isbn AS isbn,
                    b.description AS description,
                    u.user_id AS owned_by,
                    u.name AS owner_name
                FROM books AS b
                INNER JOIN users AS u USING(user_id)
                WHERE b.book_id IN (SELECT * FROM UNNEST($1::uuid[]))
                ORDER BY b.created_at DESC
            "#,
            &book_ids as _
        )
        .fetch_all(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;

        // Fetch the checkouts for the whole page in one query and key them by
        // book id, rather than querying per book.
        let book_ids = book_rows.iter().map(|r| r.book_id).collect::<Vec<BookId>>();
        let mut checkouts = self.find_checkouts(&book_ids).await?;

        let items = book_rows
            .into_iter()
            .map(|row| {
                let checkout = checkouts.remove(&row.book_id);
                row.into_book(checkout)
            })
            .collect();

        Ok(PaginatedList {
            total,
            limit,
            offset,
            items,
        })
    }

    async fn find_by_id(&self, book_id: BookId) -> AppResult<Option<Book>> {
        let row: Option<BookRow> = sqlx::query_as!(
            BookRow,
            r#"
                SELECT
                    b.book_id AS book_id,
                    b.title AS title,
                    b.author AS author,
                    b.isbn AS isbn,
                    b.description AS description,
                    u.user_id AS owned_by,
                    u.name AS owner_name
                FROM books AS b
                INNER JOIN users AS u USING(user_id)
                WHERE b.book_id = $1
            "#,
            book_id as _
        )
        .fetch_optional(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;

        match row {
            Some(r) => {
                let checkout =
                    self.find_checkouts(&[r.book_id]).await?.remove(&r.book_id);
                Ok(Some(r.into_book(checkout)))
            }
            None => Ok(None),
        }
    }

    // Only the owner can update, so the WHERE clause matches book_id and user_id.
    async fn update(&self, event: UpdateBook) -> AppResult<()> {
        let res = sqlx::query!(
            r#"
                UPDATE books
                SET
                    title = $1,
                    author = $2,
                    isbn = $3,
                    description = $4
                WHERE book_id = $5
                AND user_id = $6
            "#,
            event.title,
            event.author,
            event.isbn,
            event.description,
            event.book_id as _,
            event.requested_user as _
        )
        .execute(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;
        if res.rows_affected() < 1 {
            return Err(AppError::EntityNotFound("specified book not found".into()));
        }

        Ok(())
    }

    // Only the owner can delete, so the WHERE clause matches book_id and user_id.
    async fn delete(&self, event: DeleteBook) -> AppResult<()> {
        let res = sqlx::query!(
            r#"
                DELETE FROM books
                WHERE book_id = $1
                AND user_id = $2
            "#,
            event.book_id as _,
            event.requested_user as _
        )
        .execute(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;

        if res.rows_affected() < 1 {
            return Err(AppError::EntityNotFound("specified book not found".into()));
        }

        Ok(())
    }
}

impl BookRepositoryImpl {
    // The checkouts table holds unreturned loans only, so a hit means the book
    // is currently checked out and a miss means it is not.
    async fn find_checkouts(
        &self,
        book_ids: &[BookId],
    ) -> AppResult<HashMap<BookId, Checkout>> {
        let res = sqlx::query_as!(
            BookCheckoutRow,
            r#"
                SELECT
                    c.checkout_id,
                    c.book_id,
                    u.user_id,
                    u.name AS user_name,
                    c.checked_out_at
                FROM checkouts AS c
                INNER JOIN users AS u USING(user_id)
                WHERE c.book_id IN (SELECT * FROM UNNEST($1::uuid[]))
            "#,
            book_ids as _
        )
        .fetch_all(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?
        .into_iter()
        .map(|checkout| (checkout.book_id, Checkout::from(checkout)))
        .collect();

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::{checkout::CheckoutRepositoryImpl, user::UserRepositoryImpl};
    use kernel::{
        model::{
            checkout::event::{CreateCheckout, UpdateReturned},
            user::event::CreateUser,
        },
        repository::{checkout::CheckoutRepository, user::UserRepository},
    };
    use sqlx::types::chrono::Utc;
    use std::str::FromStr;

    // IDs written by fixtures/common.sql and fixtures/book.sql.
    const ADMIN_ID: &str = "5b4c96ac-316a-4bee-8e69-cac5eb84ff4c";
    const BOOK_ID: &str = "9890736e-a4e4-461a-a77d-eac3517ef11b";

    #[sqlx::test(fixtures("common"))]
    async fn registers_a_book(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let user_repo = UserRepositoryImpl::new(ConnectionPool::new(pool.clone()));
        let repo = BookRepositoryImpl::new(ConnectionPool::new(pool.clone()));
        let user = user_repo
            .create(CreateUser {
                name: "Test User".into(),
                email: "test@example.com".into(),
                password: "test_password".into(),
            })
            .await?;
        let book = CreateBook {
            title: "Test Title".into(),
            author: "Test Author".into(),
            isbn: "Test ISBN".into(),
            description: "Test Description".into(),
        };

        repo.create(book, user.id).await?;
        let options = BooksOptions {
            limit: 20,
            offset: 0,
        };
        let res = repo.find_all(options).await?;
        assert_eq!(res.items.len(), 1);

        let book_id = res.items[0].id;

        let Book {
            id,
            title,
            author,
            isbn,
            description,
            owner,
            ..
        } = repo
            .find_by_id(book_id)
            .await?
            .expect("the book should exist");
        assert_eq!(id, book_id);
        assert_eq!(title, "Test Title");
        assert_eq!(author, "Test Author");
        assert_eq!(isbn, "Test ISBN");
        assert_eq!(description, "Test Description");
        assert_eq!(owner.name, "Test User");

        Ok(())
    }

    #[sqlx::test(fixtures("common", "book"))]
    async fn updates_a_book(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let repo = BookRepositoryImpl::new(ConnectionPool::new(pool));
        let book_id = BookId::from_str(BOOK_ID)?;
        let book = repo.find_by_id(book_id).await?.unwrap();
        let new_author = "Updated Author";
        assert_ne!(book.author, new_author);

        repo.update(UpdateBook {
            book_id: book.id,
            title: book.title,
            author: new_author.into(),
            isbn: book.isbn,
            description: book.description,
            requested_user: UserId::from_str(ADMIN_ID)?,
        })
        .await?;

        let updated = repo.find_by_id(book_id).await?.unwrap();
        assert_eq!(updated.author, new_author);

        Ok(())
    }

    #[sqlx::test(fixtures("common", "book"))]
    async fn deletes_a_book(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let repo = BookRepositoryImpl::new(ConnectionPool::new(pool));
        let book_id = BookId::from_str(BOOK_ID)?;

        repo.delete(DeleteBook {
            book_id,
            requested_user: UserId::from_str(ADMIN_ID)?,
        })
        .await?;

        assert!(repo.find_by_id(book_id).await?.is_none());

        Ok(())
    }

    #[sqlx::test(fixtures("common", "book_list"))]
    async fn lists_books_with_pagination(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let repo = BookRepositoryImpl::new(ConnectionPool::new(pool));
        // fixtures/book_list.sql holds 50 books, newest first.
        const TOTAL: i64 = 50;

        let page = repo
            .find_all(BooksOptions {
                limit: 10,
                offset: 0,
            })
            .await?;
        assert_eq!(page.total, TOTAL);
        assert_eq!(page.items.len(), 10);
        assert_eq!(page.items[0].title, "title050");

        let page = repo
            .find_all(BooksOptions {
                limit: 10,
                offset: 10,
            })
            .await?;
        assert_eq!(page.total, TOTAL);
        assert_eq!(page.items[0].title, "title040");

        // An offset past the end returns no rows, and the total is read from them.
        let page = repo
            .find_all(BooksOptions {
                limit: 10,
                offset: 100,
            })
            .await?;
        assert_eq!(page.total, 0);
        assert!(page.items.is_empty());

        Ok(())
    }

    // The second loan has to report its own borrower, not the first one.
    async fn check_out_and_return(
        book_repo: &BookRepositoryImpl,
        checkout_repo: &CheckoutRepositoryImpl,
        book_id: BookId,
        borrower: UserId,
    ) -> anyhow::Result<()> {
        assert!(
            book_repo
                .find_by_id(book_id)
                .await?
                .unwrap()
                .checkout
                .is_none()
        );

        checkout_repo
            .create(CreateCheckout::new(book_id, borrower, Utc::now()))
            .await?;

        let book = book_repo.find_by_id(book_id).await?.unwrap();
        let checkout = book.checkout.expect("the checkout should be attached");
        assert_eq!(checkout.checked_out_by.id, borrower);

        checkout_repo
            .update_returned(UpdateReturned::new(
                checkout.checkout_id,
                book_id,
                borrower,
                Utc::now(),
            ))
            .await?;

        assert!(
            book_repo
                .find_by_id(book_id)
                .await?
                .unwrap()
                .checkout
                .is_none()
        );

        Ok(())
    }

    #[sqlx::test(fixtures("common", "book_checkout"))]
    async fn reports_the_borrower_of_the_current_checkout(
        pool: sqlx::PgPool,
    ) -> anyhow::Result<()> {
        let book_repo = BookRepositoryImpl::new(ConnectionPool::new(pool.clone()));
        let checkout_repo = CheckoutRepositoryImpl::new(ConnectionPool::new(pool));
        let book_id = BookId::from_str(BOOK_ID)?;
        let first = UserId::from_str("9582f9de-0fd1-4892-b20c-70139a7eb95b")?;
        let second = UserId::from_str("050afe56-c3da-4448-8e4d-6f44007d2ca5")?;

        check_out_and_return(&book_repo, &checkout_repo, book_id, first).await?;
        check_out_and_return(&book_repo, &checkout_repo, book_id, second).await?;

        Ok(())
    }
}
