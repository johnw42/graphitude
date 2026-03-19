use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

/// Derive `AsEnum<T>` for a data-free enum.
///
/// For each unit variant a corresponding unit struct is generated together
/// with the following impls (using `Directedness` / `Directed` as examples):
///
/// ```text
/// impl AsEnum<Directedness> for Directedness { … }   // returns *self
///
/// #[derive(Clone, Copy, Default, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
/// pub struct Directed;
///
/// impl AsEnum<Directedness> for Directed { … }
/// impl From<Directed>       for Directedness { … }
/// impl TryFrom<Directedness> for Directed    { … }
/// ```
///
/// ## Options
///
/// `#[AsEnum(arbitrary)]` — also implements `quickcheck::Arbitrary` for the
/// enum and each generated unit struct (requires `quickcheck` in the user's
/// crate).
///
/// The macro only accepts enums whose every variant carries no data.
#[proc_macro_derive(AsEnum, attributes(AsEnum))]
pub fn derive_as_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

fn expand(input: DeriveInput) -> Result<TokenStream2, syn::Error> {
    let enum_name = &input.ident;
    let vis = &input.vis;

    // Parse #[AsEnum(...)] options.
    let mut gen_arbitrary = false;
    for attr in &input.attrs {
        if attr.path().is_ident("AsEnum") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("arbitrary") {
                    gen_arbitrary = true;
                    Ok(())
                } else {
                    Err(meta.error("unknown AsEnum option"))
                }
            })?;
        }
    }

    let variants = match &input.data {
        Data::Enum(data) => &data.variants,
        _ => {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "AsEnum can only be derived for enums",
            ));
        }
    };

    // Validate: every variant must be a unit variant (no fields).
    for variant in variants {
        if !matches!(&variant.fields, Fields::Unit) {
            return Err(syn::Error::new_spanned(
                &variant.ident,
                "AsEnum only supports enums without data (unit variants only)",
            ));
        }
    }

    let variant_idents: Vec<_> = variants.iter().map(|v| &v.ident).collect();
    let n = variant_idents.len();

    // impl AsEnum<Enum> for Enum  (the enum itself is also AsEnum)
    let enum_impl = quote! {
        impl ::as_enum::AsEnum<#enum_name> for #enum_name {
            fn as_enum(&self) -> #enum_name {
                *self
            }
        }
    };

    // Optional: impl quickcheck::Arbitrary for the enum.
    let enum_arbitrary = if gen_arbitrary {
        let arms = variant_idents.iter().enumerate().map(|(i, variant)| {
            quote! { #i => #enum_name::#variant }
        });
        quote! {
            impl ::quickcheck::Arbitrary for #enum_name {
                fn arbitrary(g: &mut ::quickcheck::Gen) -> Self {
                    match <usize as ::quickcheck::Arbitrary>::arbitrary(g) % #n {
                        #(#arms,)*
                        _ => unreachable!(),
                    }
                }
            }
        }
    } else {
        quote! {}
    };

    // For each variant: struct + four impls + optional Arbitrary.
    let variant_items: TokenStream2 = variant_idents
        .iter()
        .map(|variant| {
            let arbitrary_impl = if gen_arbitrary {
                quote! {
                    impl ::quickcheck::Arbitrary for #variant {
                        fn arbitrary(_g: &mut ::quickcheck::Gen) -> Self {
                            #variant
                        }
                    }
                }
            } else {
                quote! {}
            };

            quote! {
                #[derive(Clone, Copy, Default, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
                #vis struct #variant;

                impl ::as_enum::AsEnum<#enum_name> for #variant {
                    fn as_enum(&self) -> #enum_name {
                        #enum_name::#variant
                    }
                }

                impl ::core::convert::From<#variant> for #enum_name {
                    fn from(_: #variant) -> Self {
                        #enum_name::#variant
                    }
                }

                impl ::core::convert::TryFrom<#enum_name> for #variant {
                    type Error = ();

                    fn try_from(value: #enum_name) -> ::core::result::Result<Self, Self::Error> {
                        match value {
                            #enum_name::#variant => ::core::result::Result::Ok(#variant),
                            _ => ::core::result::Result::Err(()),
                        }
                    }
                }

                #arbitrary_impl
            }
        })
        .collect();

    Ok(quote! {
        #enum_impl
        #enum_arbitrary
        #variant_items
    })
}
