use darling::{FromDeriveInput, FromField};
use proc_macro::TokenStream;
use quote::quote;

#[derive(FromDeriveInput)]
#[darling(attributes(branded), supports(struct_newtype))]
pub(crate) struct BrandedTypeOptions {
    ident: syn::Ident,
    data: darling::ast::Data<(), BrandedFieldOptions>,

    #[darling(default)]
    serde: bool,
    #[darling(default)]
    uuid: bool,
    #[darling(default)]
    sqlx: bool,
}

#[derive(FromField)]
pub(crate) struct BrandedFieldOptions {
    ty: syn::Type,
}

#[proc_macro_derive(Branded, attributes(branded))]
pub fn branded_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input);
    let options = match BrandedTypeOptions::from_derive_input(&input) {
        Ok(options) => options,
        Err(err) => return err.write_errors().into(),
    };
    let expanded = match expand_branded_derive(options) {
        Ok(expanded) => expanded,
        Err(err) => return err.to_compile_error().into(),
    };
    expanded.into()
}

pub(crate) fn expand_branded_derive(
    options: BrandedTypeOptions,
) -> syn::Result<proc_macro2::TokenStream> {
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
    let constructor_doc_comment = format!("Construct a new `{struct_name}` value.");
    tokens.extend(quote! {
        impl Branded for #struct_name {
            type Inner = #ty;
            fn inner(&self) -> &#ty { &self.0 }
            fn into_inner(self) -> #ty { self.0 }
        }
        impl #struct_name {
            #[doc = #constructor_doc_comment]
            pub fn new(inner: #ty) -> Self { Self(inner) }
        }
    });

    tokens.extend(expand_clone_copy_impl(struct_name));
    tokens.extend(expand_debug_display_impl(struct_name));
    tokens.extend(expand_default_impl(struct_name));
    tokens.extend(expand_ord_impl(struct_name));
    tokens.extend(expand_hash_impl(struct_name));

    if options.serde {
        tokens.extend(expand_serde_impl(struct_name));
    }

    if options.sqlx {
        tokens.extend(expand_sqlx_impl(struct_name));
    }

    if options.uuid {
        tokens.extend(expand_uuid_impl(struct_name));
    }

    Ok(tokens)
}

/// Derive a Clone implementation for the branded type if the inner type is Clone.
pub(crate) fn expand_clone_copy_impl(brand_struct_name: &syn::Ident) -> proc_macro2::TokenStream {
    let copy_trait: syn::Path = syn::parse_quote!(::std::marker::Copy);
    let clone_trait: syn::Path = syn::parse_quote!(::std::clone::Clone);
    quote! {
        impl #clone_trait for #brand_struct_name
        where
            for<'__branded> <Self as Branded>::Inner: #clone_trait,
        {
            fn clone(&self) -> Self {
                Self::new(self.inner().clone())
            }
        }
        impl #copy_trait for #brand_struct_name
        where
            for<'__branded> <Self as Branded>::Inner: #copy_trait,
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
) -> proc_macro2::TokenStream {
    let display_trait: syn::Path = syn::parse_quote!(::std::fmt::Display);
    let debug_trait: syn::Path = syn::parse_quote!(::std::fmt::Debug);
    quote! {
        impl #display_trait for #brand_struct_name
        where
            for<'__branded> <Self as Branded>::Inner: #display_trait,
        {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                ::std::fmt::Display::fmt(&self.inner(), f)
            }
        }
        impl #debug_trait for #brand_struct_name
        where
            for<'__branded> <Self as Branded>::Inner: #debug_trait,
        {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.debug_tuple(stringify!(#brand_struct_name)).field(self.inner()).finish()
            }
        }
    }
}

/// Derive a Default implementation for the branded type if the inner type conforms to Default.
pub(crate) fn expand_default_impl(brand_struct_name: &syn::Ident) -> proc_macro2::TokenStream {
    let path: syn::Path = syn::parse_quote!(::std::default::Default);
    quote! {
        impl #path for #brand_struct_name
        where
            for<'__branded> <Self as Branded>::Inner: #path,
        {
            fn default() -> Self {
                Self::new(<Self as Branded>::Inner::default())
            }
        }
    }
}

/// Derive a PartialEq, Eq, Ord, and PartialOrd implementation for the branded type if the inner
/// type conforms to any of those traits.
pub(crate) fn expand_ord_impl(brand_struct_name: &syn::Ident) -> proc_macro2::TokenStream {
    let eq_trait: syn::Path = syn::parse_quote!(::std::cmp::Eq);
    let partial_eq_trait: syn::Path = syn::parse_quote!(::std::cmp::PartialEq);
    let ord_trait: syn::Path = syn::parse_quote!(::std::cmp::Ord);
    let partial_ord_trait: syn::Path = syn::parse_quote!(::std::cmp::PartialOrd);
    quote! {
        impl #partial_eq_trait for #brand_struct_name
        where
            for<'__branded> <Self as Branded>::Inner: #partial_eq_trait,
        {
            fn eq(&self, other: &Self) -> bool {
                self.inner().eq(other.inner())
            }
        }
        impl #eq_trait for #brand_struct_name
        where
            for<'__branded> <Self as Branded>::Inner: #eq_trait,
        {
        }
        impl #ord_trait for #brand_struct_name
        where
            for<'__branded> <Self as Branded>::Inner: #ord_trait,
        {
            fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
                self.0.cmp(&other.0)
            }
        }
        impl #partial_ord_trait for #brand_struct_name
        where
            for<'__branded> <Self as Branded>::Inner: #partial_ord_trait,
        {
            fn partial_cmp(&self, other: &Self) -> ::std::option::Option<::std::cmp::Ordering> {
                self.0.partial_cmp(&other.0)
            }
        }
    }
}

/// Derive a Hash implementation for the branded type if the inner type conforms to Hash.
pub(crate) fn expand_hash_impl(brand_struct_name: &syn::Ident) -> proc_macro2::TokenStream {
    let hash_trait: syn::Path = syn::parse_quote!(::std::hash::Hash);
    quote! {
        impl #hash_trait for #brand_struct_name
        where
            for<'__branded> <Self as Branded>::Inner: #hash_trait,
        {
            fn hash<H: ::std::hash::Hasher>(&self, state: &mut H) {
                self.inner().hash(state);
            }
        }
    }
}

/// Derive a Serde implementation for the branded type if asked for.
pub(crate) fn expand_serde_impl(brand_struct_name: &syn::Ident) -> proc_macro2::TokenStream {
    let serialize_trait: syn::Path = syn::parse_quote!(::serde::Serialize);
    let deserialize_trait: syn::Path = syn::parse_quote!(::serde::Deserialize);
    quote! {
        impl #serialize_trait for #brand_struct_name
        where
            for<'__branded> <Self as Branded>::Inner: #serialize_trait,
        {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                self.inner().serialize(serializer)
            }
        }

        impl<'de> #deserialize_trait<'de> for #brand_struct_name
        where
            for<'__branded> <Self as Branded>::Inner: #deserialize_trait<'de>,
        {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                <Self as Branded>::Inner::deserialize(deserializer)
                    .map(Self::new)
            }
        }
    }
}

/// Derive a sqlx Type, Encode, and Decode implementation for the branded type if asked for.
pub(crate) fn expand_sqlx_impl(brand_struct_name: &syn::Ident) -> proc_macro2::TokenStream {
    let type_trait: syn::Path = syn::parse_quote!(::sqlx::Type);
    let encode_trait: syn::Path = syn::parse_quote!(::sqlx::Encode);
    let decode_trait: syn::Path = syn::parse_quote!(::sqlx::Decode);
    quote! {
        impl<DB> #type_trait<DB> for #brand_struct_name
        where
            for<'__branded> <Self as Branded>::Inner: #type_trait<DB>,
            DB: ::sqlx::Database,
        {
            fn type_info() -> DB::TypeInfo {
                <Self as Branded>::Inner::type_info()
            }
        }

        impl<'de, DB> #decode_trait<'de, DB> for #brand_struct_name
        where
            for<'__branded> Self: Branded,
            <Self as Branded>::Inner: for<'a> #decode_trait<'a, DB>,
            DB: ::sqlx::Database,
        {
            fn decode(value: DB::ValueRef<'_>) -> ::std::result::Result<#brand_struct_name, ::sqlx::error::BoxDynError> {
                <Self as Branded>::Inner::decode(value).map(Self::new)
            }
        }

        impl<'en, DB> #encode_trait<'en, DB> for #brand_struct_name
        where
            for<'__branded> Self: Branded,
            <Self as Branded>::Inner: for<'a> #encode_trait<'a, DB>,
            DB: ::sqlx::Database,
        {
            fn encode_by_ref(&self, buf: &mut DB::ArgumentBuffer<'_>) -> ::std::result::Result<::sqlx::encode::IsNull, ::sqlx::error::BoxDynError> {
                self.inner().encode_by_ref(buf)
            }
        }
    }
}

pub(crate) fn expand_uuid_impl(brand_struct_name: &syn::Ident) -> proc_macro2::TokenStream {
    quote! {
        impl #brand_struct_name
        where
            for<'__branded> Self: Branded<Inner = ::uuid::Uuid>
        {
            /// Get the nil UUID.
            fn nil() -> Self { Self::new(::uuid::Uuid::nil()) }

            /// Get a new random UUID v4.
            fn new_v4() -> Self { Self::new(::uuid::Uuid::new_v4()) }
        }
    }
}
