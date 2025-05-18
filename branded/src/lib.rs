pub use branded_derive::Branded;

pub trait Branded {
    type Inner;
    fn inner(&self) -> &Self::Inner;
    fn into_inner(self) -> Self::Inner;
}

#[derive(Branded)]
#[branded]
pub struct UserId(String);
