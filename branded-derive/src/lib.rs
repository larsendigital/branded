use darling::{FromDeriveInput, FromField};
use proc_macro::TokenStream;
use quote::quote;

#[derive(FromDeriveInput)]
#[darling(supports(struct_newtype))]
pub(crate) struct BrandedTypeOptions {
    serde: Option<syn::Ident>,
    uuid: Option<syn::Ident>,
    sqlx: Option<syn::Ident>,
    ident: syn::Ident,
    data: darling::ast::Data<(), BrandedFieldOptions>,
}

#[derive(FromField)]
#[darling(attributes(branded))]
pub(crate) struct BrandedFieldOptions {
    ty: syn::Type,
}

#[proc_macro_derive(Branded)]
pub fn branded(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input);
    let options = match BrandedTypeOptions::from_derive_input(&input) {
        Ok(options) => options,
        Err(err) => return err.write_errors().into(),
    };
    let expanded = match expand_branded(options) {
        Ok(expanded) => expanded,
        Err(err) => return err.to_compile_error().into(),
    };
    expanded.into()
}

pub(crate) fn expand_branded(options: BrandedTypeOptions) -> syn::Result<proc_macro2::TokenStream> {
    let mut tokens = proc_macro2::TokenStream::new();
    let struct_name = &options.ident;
    let field = options
        .data
        .take_struct()
        .map(|fields| {
            fields.into_iter().next().ok_or(syn::Error::new(
                struct_name.span(),
                "struct must have exactly one field (newtype pattern)",
            ))
        })
        .transpose()?
        .ok_or(syn::Error::new(
            struct_name.span(),
            "derive(Branded) can only be used on structs",
        ))?;
    let ty = field.ty;
    let struct_trait_name = quote::format_ident!("{}Brand", struct_name);
    let struct_doc_comment = format!("The `{struct_name}` branded type. This utility trait provides blanket implementations for common traits.");
    let constructor_doc_comment = format!("Construct a new `{struct_name}` value.");
    tokens.extend(quote! {
        #[doc = #struct_doc_comment]
        trait #struct_trait_name {
            type Inner;
        }
        impl #struct_name {
            #[doc = #constructor_doc_comment]
            pub fn new(inner: #ty) -> Self { Self(inner) }

            /// Get the underlying type.
            pub fn inner(&self) -> &#ty { &self.0 }

            /// Convert this branded type into the underlying type.
            pub fn into_inner(self) -> #ty { self.0 }
        }
        impl #struct_trait_name for #struct_name {
            type Inner = #ty;
        }
    });

    tokens.extend(expand_clone_copy_impl(struct_name, &struct_trait_name));
    tokens.extend(expand_debug_display_impl(struct_name, &struct_trait_name));
    tokens.extend(expand_default_impl(struct_name, &struct_trait_name));
    tokens.extend(expand_ord_impl(struct_name, &struct_trait_name));
    tokens.extend(expand_hash_impl(struct_name, &struct_trait_name));

    Ok(tokens)
}

/// Derive a Clone implementation for the branded type if the inner type is Clone.
pub(crate) fn expand_clone_copy_impl(
    brand_struct_name: &syn::Ident,
    brand_struct_trait_name: &syn::Ident,
) -> proc_macro2::TokenStream {
    let copy_trait: syn::Path = syn::parse_quote!(::std::marker::Copy);
    let clone_trait: syn::Path = syn::parse_quote!(::std::clone::Clone);
    quote! {
        impl #clone_trait for #brand_struct_name
        where
            for<'__branded> <Self as #brand_struct_trait_name>::Inner: #clone_trait,
        {
            fn clone(&self) -> Self {
                Self::new(self.inner().clone())
            }
        }
        impl #copy_trait for #brand_struct_name
        where
            for<'__branded> <Self as #brand_struct_trait_name>::Inner: #clone_trait,
        {
        }
    }
}

/// Derive a Display and Debug implementation for the branded type if the inner type conforms to
/// either trait.
///
/// For the Debug implementation, this generates a Debug implementation that prints a tuple of the
/// inner type contained in the branded type name.
pub(crate) fn expand_debug_display_impl(
    brand_struct_name: &syn::Ident,
    brand_struct_trait_name: &syn::Ident,
) -> proc_macro2::TokenStream {
    let display_trait: syn::Path = syn::parse_quote!(::std::fmt::Display);
    let debug_trait: syn::Path = syn::parse_quote!(::std::fmt::Debug);
    quote! {
        impl #display_trait for #brand_struct_name
        where
            for<'__branded> <Self as #brand_struct_trait_name>::Inner: #display_trait,
        {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                ::std::fmt::Display::fmt(&self.inner(), f)
            }
        }
        impl #debug_trait for #brand_struct_name
        where
            for<'__branded> <Self as #brand_struct_trait_name>::Inner: #debug_trait,
        {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.debug_tuple(stringify!(#brand_struct_name)).field(self.inner()).finish()
            }
        }
    }
}

/// Derive a Default implementation for the branded type if the inner type conforms to Default.
pub(crate) fn expand_default_impl(
    brand_struct_name: &syn::Ident,
    brand_struct_trait_name: &syn::Ident,
) -> proc_macro2::TokenStream {
    let path: syn::Path = syn::parse_quote!(::std::default::Default);
    quote! {
        impl #path for #brand_struct_name
        where
            for<'__branded> <Self as #brand_struct_trait_name>::Inner: #path,
        {
            fn default() -> Self {
                Self::new(<Self as #brand_struct_trait_name>::Inner::default())
            }
        }
    }
}

/// Derive a PartialEq, Eq, Ord, and PartialOrd implementation for the branded type if the inner
/// type conforms to any of those traits.
pub(crate) fn expand_ord_impl(
    brand_struct_name: &syn::Ident,
    brand_struct_trait_name: &syn::Ident,
) -> proc_macro2::TokenStream {
    let eq_trait: syn::Path = syn::parse_quote!(::std::cmp::Eq);
    let partial_eq_trait: syn::Path = syn::parse_quote!(::std::cmp::PartialEq);
    let ord_trait: syn::Path = syn::parse_quote!(::std::cmp::Ord);
    let partial_ord_trait: syn::Path = syn::parse_quote!(::std::cmp::PartialOrd);
    quote! {
        impl #partial_eq_trait for #brand_struct_name
        where
            for<'__branded> <Self as #brand_struct_trait_name>::Inner: #partial_eq_trait,
        {
            fn eq(&self, other: &Self) -> bool {
                self.inner().eq(other.inner())
            }
        }
        impl #eq_trait for #brand_struct_name
        where
            for<'__branded> <Self as #brand_struct_trait_name>::Inner: #eq_trait,
        {
        }
        impl #ord_trait for #brand_struct_name
        where
            for<'__branded> <Self as #brand_struct_trait_name>::Inner: #ord_trait,
        {
            fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
                self.inner().cmp(&other.inner())
            }
        }
        impl #partial_ord_trait for #brand_struct_name
        where
            for<'__branded> <Self as #brand_struct_trait_name>::Inner: #partial_ord_trait,
        {
            fn partial_cmp(&self, other: &Self) -> ::std::option::Option<::std::cmp::Ordering> {
                self.inner().partial_cmp(&other.inner())
            }
        }
    }
}

/// Derive a Hash implementation for the branded type if the inner type conforms to Hash.
pub(crate) fn expand_hash_impl(
    brand_struct_name: &syn::Ident,
    brand_struct_trait_name: &syn::Ident,
) -> proc_macro2::TokenStream {
    let hash_trait: syn::Path = syn::parse_quote!(::std::hash::Hash);
    quote! {
        impl #hash_trait for #brand_struct_name
        where
            for<'__branded> <Self as #brand_struct_trait_name>::Inner: #hash_trait,
        {
            fn hash<H: ::std::hash::Hasher>(&self, state: &mut H) {
                self.inner().hash(state);
            }
        }
    }
}
