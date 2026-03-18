/// A trait implemented by both an enum and unit structs that represent its
/// variants, allowing any of them to be converted to the enum value.
///
/// The [`AsEnum`] derive macro implements this trait automatically for enums
/// without data, generating a unit struct per variant plus all the required
/// trait impls.
pub trait AsEnum<T>: Clone {
    fn as_enum(&self) -> T;
}

pub use as_enum_derive::AsEnum;
