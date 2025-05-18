# Branded

Branded types for Rust.

> This crate is a continuation of the now-archived [bty](https://github.com/lffg/bty) crate. It has been rewritten to be
> a derive macro, and to support SQLx 0.8.

Branded types are types that have a unique brand attached to them. They are particularly useful for managing ID types in
large domains where it is easy to confuse the IDs of different domain objects. With Rust's nominal typing, branded types
makes it impossible to confuse the IDs.

Thanks to Rust's trait system, we can transparently derive traits for our branded types based on the inner type, making
them completely transparent to other libraries such as `serde`, and `sqlx`. This means that the brand types derive
common traits such as `Copy`, `Clone`, `Debug`, `Eq`, `PartialEq`, `Ord`, `PartialOrd`, `Default`, and `Hash`, if the
inner type implements them.

The crate provides the `Branded` trait and the `Branded` derive macro.

```toml
# Cargo.toml

# The `serde` feature transparently derives the `Serialize` and `Deserialize` traits for the branded type.
# The `sqlx` feature derives the `Type`, `Encode`, and `Decode` traits for the branded type.
# The `uuid` feature exposes `nil()` and `new_v4()` methods on the branded type.
[dependencies]
branded = { version = "0.1", features = ["serde", "sqlx", "uuid"] }
```

## Example

```rust
use branded::Branded;

// It is now impossible to confuse a UserId with a LogRecordId.
#[derive(Branded)]
pub struct UserId(String);

fn foo() {
    let user = UserId::new("123456".to_owned());
    // Get a reference to the inner type
    let user_id = user.inner();
    // Convert the branded type to the inner type
    let user_id = user.into_inner();
}
```

## Serde

The `serde` feature transparently derives the `Serialize` and `Deserialize` traits for the branded type. Pass `serde` as
an option to the `Branded` derive macro to enable this feature.

```rust
use branded::Branded;

#[derive(Branded)]
#[branded(serde)]
pub struct UserId(String);
```

## SQLx

The `sqlx` feature derives the `Type`, `Encode`, and `Decode` traits for the branded type. Pass `sqlx` as an option to
the`Branded` derive macro to enable this feature.

```rust
use branded::Branded;

#[derive(Branded)]
#[branded(sqlx)]
pub struct LogRecordId(i64);
```

## UUID

The `uuid` feature exposes `nil()` and `new_v4()` methods on the branded type. Pass `uuid` as an option to the `Branded`
derive macro to enable this feature.

```rust
use branded::Branded;

#[derive(Branded)]
#[branded(uuid)]
pub struct AuditLogEntryId(uuid::Uuid);
```

## License

Licensed under the [MIT License](LICENSE).
