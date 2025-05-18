//! # Branded
//!
//! Branded types for Rust.
//!
//! Branded types are types that have a unique brand attached to them. They are particularly useful
//! for managing ID types in large domains where it is easy to confuse the IDs of different domain
//! objects. With Rust's nominal typing, branded types makes it impossible to confuse the IDs.
//!
//! Thanks to Rust's trait system, we can transparently derive traits for our branded types based on
//! the inner type, making them completely transparent to other libraries such as `serde`, and
//! `sqlx`.
//!
//! > This crate is a continuation of the now-archived [bty](https://github.com/lffg/bty) crate. It
//! > has been rewritten to be
//! > a derive macro, and to support SQLx 0.8.
//!
//! The crate provides the `Branded` trait and the `Branded` derive macro.
//!
//! ```
//! use branded::Branded;
//!
//! #[derive(Branded)]
//! pub struct UserId(String);
//! ```
//!
//! ## Serde
//!
//! The `serde` feature transparently derives the `Serialize` and `Deserialize` traits for the
//! branded type. Pass `serde` as an option to the `Branded` derive macro to enable this feature.
//!
//! ```
//! use branded::Branded;
//!
//! #[derive(Branded)]
//! #[branded(serde)]
//! pub struct UserId(String);
//! ```
//!
//! ## SQLx
//!
//! The `sqlx` feature derives the `Type`, `Encode`, and `Decode` traits for the branded type. Pass
//! `sqlx` as an option to the `Branded` derive macro to enable this feature.
//!
//! ```
//! use branded::Branded;
//!
//! #[derive(Branded)]
//! #[branded(sqlx)]
//! pub struct UserId(String);
//! ```
//!
//! ## UUID
//!
//! The `uuid` feature exposes `nil()` and `new_v4()` methods on the branded type. Pass `uuid` as an
//! option to the `Branded` derive macro to enable this feature.
//!
//! ```
//! use branded::Branded;
//!
//! #[derive(Branded)]
//! #[branded(uuid)]
//! pub struct UserId(uuid::Uuid);
//! ```

pub use branded_derive::Branded;

/// A trait for types that are a brand of some inner type.
///
/// This trait is not used for specific features internally, but you may use it if you want to write
/// generic code over all branded types.
pub trait Branded {
    /// The inner type of the brand.
    ///
    /// This type is usually best represented as an ID such as string, an integer, or a UUID, but
    /// any type is allowed.
    type Inner;

    /// Get a reference to the inner type.
    fn inner(&self) -> &Self::Inner;

    /// Convert the branded type to the inner type.
    fn into_inner(self) -> Self::Inner;
}
