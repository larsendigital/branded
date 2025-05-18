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
