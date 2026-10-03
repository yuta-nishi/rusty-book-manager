// Clippy 1.99 flags `redundant_field_names` inside `#[derive(new)]` expansions,
// which emit `Self { field: field }`.
// TODO: drop this allow once either of these lands:
// - https://github.com/rust-lang/rust-clippy/pull/17700 (false positive fix)
// - https://github.com/nrc/derive-new/pull/72 (stops emitting `field: field`)
#![allow(clippy::redundant_field_names)]

pub mod model;
pub mod repository;
