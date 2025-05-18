use branded::Branded;
use std::fmt::{Debug, Display};
use std::hash::Hash;

#[test]
fn conforms_to_inner_traits() {
    #[derive(Branded)]
    pub struct UserId(u32);

    fn needs_clone<T: Clone>() {}
    fn needs_copy<T: Copy>() {}
    fn needs_default<T: Default>() {}
    fn needs_debug<T: Debug>() {}
    fn needs_display<T: Display>() {}
    fn needs_eq<T: PartialEq>() {}
    fn needs_hash<T: Hash>() {}
    fn needs_ord<T: PartialOrd>() {}

    needs_clone::<UserId>();
    needs_copy::<UserId>();
    needs_default::<UserId>();
    needs_debug::<UserId>();
    needs_display::<UserId>();
    needs_eq::<UserId>();
    needs_hash::<UserId>();
    needs_ord::<UserId>();
}

#[test]
fn test_accessors() {
    #[derive(Branded)]
    pub struct UserId(u32);

    let user_id = UserId::new(123);
    assert_eq!(user_id.inner(), &123);
    assert_eq!(user_id.into_inner(), 123);
}

#[cfg(feature = "serde")]
mod serde {
    use branded::Branded;
    use serde::de::DeserializeOwned;
    use serde::Serialize;

    #[test]
    fn test_serde_derive() {
        #[derive(Branded)]
        #[branded(serde)]
        pub struct UserId(String);

        fn needs_serialize<T: Serialize>() {}
        fn needs_deserialize<T: DeserializeOwned>() {}

        needs_serialize::<UserId>();
        needs_deserialize::<UserId>();

        let id = UserId::new("123".to_string());
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, r#""123""#);
        let recovered: UserId = serde_json::from_str(&json).unwrap();
        assert_eq!(recovered, id);
    }
}

#[cfg(feature = "sqlx")]
mod sqlx {
    use branded::Branded;
    use sqlx::Database;

    #[test]
    fn test_sqlx_derive() {
        #[derive(Branded)]
        #[branded(sqlx)]
        pub struct UserId(String);

        fn needs_type<T: sqlx::Type<DB>, DB: Database>() {}
        fn needs_encode<'en, T: sqlx::Encode<'en, DB>, DB: Database>() {}
        fn needs_decode<'de, T: sqlx::Decode<'de, DB>, DB: Database>() {}

        needs_type::<UserId, sqlx::Sqlite>();
        needs_encode::<UserId, sqlx::Sqlite>();
        needs_decode::<UserId, sqlx::Sqlite>();
    }
}
