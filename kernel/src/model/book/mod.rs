use super::{
    id::{BookId, CheckoutId},
    user::{BookOwner, CheckoutUser},
};
use chrono::{DateTime, Utc};

pub mod event;

#[derive(Debug)]
pub struct Book {
    pub id: BookId,
    pub title: String,
    pub author: String,
    pub isbn: String,
    pub description: String,
    pub owner: BookOwner,
    pub checkout: Option<Checkout>,
}

#[derive(Debug)]
pub struct BooksOptions {
    pub limit: i64,
    pub offset: i64,
}

// Checkout information attached to a book. This is a different type from
// model::checkout::Checkout, which is the aggregate used by the checkout APIs.
#[derive(Debug)]
pub struct Checkout {
    pub checkout_id: CheckoutId,
    pub checked_out_by: CheckoutUser,
    pub checked_out_at: DateTime<Utc>,
}
