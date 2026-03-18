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
/// The macro only accepts enums whose every variant carries no data.
#[proc_macro_derive(AsEnum)]
pub fn derive_as_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

fn expand(input: DeriveInput) -> Result<TokenStream2, syn::Error> {
    let enum_name = &input.ident;
    let vis = &input.vis;

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

    // impl AsEnum<Enum> for Enum  (the enum itself is also AsEnum)
    let enum_impl = quote! {
        impl ::as_enum::AsEnum<#enum_name> for #enum_name {
            fn as_enum(&self) -> #enum_name {
                *self
            }
        }
    };

    // For each variant: struct + four impls
    let variant_items: TokenStream2 = variant_idents
        .iter()
        .map(|variant| {
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
            }
        })
        .collect();

    Ok(quote! {
        #enum_impl
        #variant_items
    })
}
