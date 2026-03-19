//! # Overview
//!
//! This crate provides a way to implement dependently-type enums using the
//! [`AsEnum`] derive macro, allowing the variant of an enum to be treated as a
//! type in its own right by generating unit structs for each variant and
//! implementing `AsEnum` for both the enum and the unit structs.
//!
//! # Example
//!
//! For example, consider this code:
//!
//! ```
//! use as_enum::AsEnum;
//!
//! #[derive(AsEnum, Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
//! enum MyEnum {
//!   VariantA,
//!   VariantB,
//! }
//! ```
//!
//! This will generate the following unit structs:
//! ```ignore
//! struct VariantA;
//! struct VariantB;
//! ```
//!
//! Each of [`MyEnum`], [`VariantA`], and [`VariantB`] will implement
//! `AsEnum<MyEnum>`, allowing any of them to be converted to the enum value.
//! (`MyEnum::as_enum` is an identity function, while `VariantA::as_enum` and
//! `VariantB::as_enum` return `MyEnum::VariantA` and `MyEnum::VariantB`,
//! respectively.)
//!
//! This pattern allows for compile-time specialization of behavior based on the
//! enum variant.  Consider a type `MyType<E: AsEnum<MyEnum>>` that behaves
//! differently based on the variant of `MyEnum`:
//!
//! ```ignore
//! struct MyType<E: AsEnum<MyEnum>> {
//!   enum_value: E,
//! }
//!
//! impl<E: AsEnum<MyEnum>> MyType<E> {
//!   fn print_enum_variant(enum_value: E) -> Self {
//!     println!("Enum variant: {:?}", enum_value.as_enum());
//!   }
//! }
//!
//! impl MyType<VariantA> {
//!   fn only_for_variant_a(&self) { /* ... */ }
//! }
//! ```
//!
//! In this example, `MyType` can be instantiated as `MyType<MyEnum>`, allowing
//! its behavior to be determined at runtime based on the enum variant, or as
//! `MyType<VariantA>`, allowing for compile-time specialization of behavior
//! specific to `VariantA`.
//!
//! # Usage
//!
//! To use the `AsEnum` derive macro, simply add `#[derive(AsEnum)]` to your
//! enum definition, and ensure the enum also implements `Clone`, `Copy`,
//! `Debug`, `Hash`, `PartialEq`, `Eq`, `PartialOrd`, and `Ord` (as required by
//! the definition of the `AsEnum` trait).)
//!
//! # QuickCheck Integration
//!
//! The enum and its associated unit structs can also be make to implement
//! `quickcheck::Arbitrary` by adding `#[AsEnum(arbitrary)]` to the enum
//! definition, which will generate an implementation of `Arbitrary` for the
//! enum and each of the unit structs. The implementation for the enum will
//! generate arbitrary values by randomly choosing one of the variants, while
//! the implementation for each unit struct will always generate that unit
//! struct. (This requires `quickcheck` in the user's crate.)
//!
//! # Adding Methods
//!
//! The following pattern is useful to add methods to the enum and its
//! corresponding unit structs:
//!
//! ```ignore
//! pub trait MyEnumTrait: AsEnum<MyEnum> {
//!     fn name(&self) -> bool {
//!         match self.as_enum() {
//!             MyEnum::VariantA => "VariantA",
//!             MyEnum::VariantB => "VariantB",
//!         }
//!     }
//! }
//!
//! impl<T> MyEnumTrait for T where T: AsEnum<MyEnum> {}
//! ```
use std::{fmt::Debug, hash::Hash};

pub use as_enum_derive::AsEnum;

/// The trait implemented by the `AsEnum` derive macro. See the crate-level documentation for details and examples.
pub trait AsEnum<T>:
    Clone + Copy + Debug + Hash + PartialEq + Eq + PartialOrd + Ord + Into<T> + TryFrom<T> + Send + Sync
{
    fn as_enum(&self) -> T;
}
