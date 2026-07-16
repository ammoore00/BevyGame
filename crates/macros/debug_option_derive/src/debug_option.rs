use crate::{compile_error, compile_error_spanned};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use syn::spanned::Spanned;
use syn::{parse_macro_input, DeriveInput, Fields, FieldsNamed, FieldsUnnamed, Type};

pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let syn::Data::Struct(ref data) = input.data else {
        return compile_error_spanned(&input.ident, DebugOptionDeriveErrorKind::NonStruct);
    };

    let field_access = match data.fields {
        Fields::Named(ref fields) => named_fields(&input, fields),
        Fields::Unnamed(ref fields) => unnamed_fields(&input, fields),
        Fields::Unit => return compile_error_spanned(&input.ident, DebugOptionDeriveErrorKind::NoField),
    };

    let field_access = match field_access {
        Ok(f) => f,
        Err(e) => return compile_error(e.span, e.kind),
    };

    // Grab generics so we can implement the trait for generic structs too
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let res_name = format_ident!("{}Res", name);

    let expanded = quote! {
        impl #impl_generics ::common::dev_tools::DebugOption for #name #ty_generics #where_clause {
            type Res = #res_name;

            fn get(&self) -> bool {
                self.#field_access
            }

            fn set(&mut self, value: bool) {
                self.#field_access = value;
            }
        }

        #[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct #res_name(bool);
        impl DebugState for #res_name {
            fn get(&self) -> bool {
                self.0
            }

            fn set(&mut self, value: bool) {
                self.0 = value;
            }
        }
    };

    TokenStream::from(expanded)
}

fn named_fields(input: &DeriveInput, fields: &FieldsNamed) -> Result<proc_macro2::TokenStream, DebugOptionDeriveError> {
    // Look for a bool field called "enabled"
    if let Some(field) = fields
        .named
        .iter()
        .find(|f| f.ident.as_ref().unwrap() == "enabled")
    {
        if !is_bool(&field.ty) {
            return Err(DebugOptionDeriveError::new(field.span(), DebugOptionDeriveErrorKind::FieldNotBool));
        }
        let ident = field.ident.as_ref().unwrap();
        Ok(quote!(#ident))
    } else {
        // Otherwise require #[enabled] attribute
        let enabled_fields: Vec<_> = fields
            .named
            .iter()
            .filter(|f| has_enabled_attr(f))
            .collect();

        match enabled_fields.len() {
            0 => Err(DebugOptionDeriveError::new(input.ident.span(), DebugOptionDeriveErrorKind::MissingEnabledAttr)),
            1 => {
                let field = enabled_fields[0];
                if !is_bool(&field.ty) {
                    return Err(DebugOptionDeriveError::new(field.span(), DebugOptionDeriveErrorKind::FieldNotBool));
                }
                let ident = field.ident.as_ref().unwrap();
                Ok(quote!(#ident))
            }
            _ => Err(DebugOptionDeriveError::new(input.ident.span(), DebugOptionDeriveErrorKind::MultipleEnabledAttrs)),
        }
    }
}

fn unnamed_fields(input: &DeriveInput, fields: &FieldsUnnamed) -> Result<proc_macro2::TokenStream, DebugOptionDeriveError> {
    // Tuple struct with only one field
    if fields.unnamed.len() == 1 {
        let field = fields.unnamed.first().unwrap();
        if !is_bool(&field.ty) {
            return Err(DebugOptionDeriveError::new(field.span(), DebugOptionDeriveErrorKind::FieldNotBool));
        }
        Ok(quote!(0))
    } else {
        // Tuple struct with multiple fields, require #[enabled]
        let enabled_fields: Vec<_> = fields
            .unnamed
            .iter()
            .enumerate()
            .filter(|(_, f)| has_enabled_attr(f))
            .collect();

        match enabled_fields.len() {
            0 => Err(DebugOptionDeriveError::new(input.ident.span(), DebugOptionDeriveErrorKind::MissingEnabledAttr)),
            1 => {
                let (idx, field) = enabled_fields[0];
                if !is_bool(&field.ty) {
                    return Err(DebugOptionDeriveError::new(field.span(), DebugOptionDeriveErrorKind::FieldNotBool));
                }
                let index = syn::Index::from(idx);
                Ok(quote!(#index))
            }
            _ => Err(DebugOptionDeriveError::new(input.ident.span(), DebugOptionDeriveErrorKind::MultipleEnabledAttrs)),
        }
    }
}

//------ Helper Functions ------//

fn has_enabled_attr(field: &syn::Field) -> bool {
    // In syn 2.x, checking the path of the attribute meta
    field
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("enabled"))
}

fn is_bool(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty
        && type_path.path.is_ident("bool")
    {
        return true;
    }
    false
}

#[derive(Debug, derive_new::new)]
struct DebugOptionDeriveError {
    span: proc_macro2::Span,
    kind: DebugOptionDeriveErrorKind,
}
impl Display for DebugOptionDeriveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}
impl Error for DebugOptionDeriveError {}

#[derive(thiserror::Error, Debug)]
enum DebugOptionDeriveErrorKind {
    #[error("DebugOption can only be derived for structs")]
    NonStruct,
    #[error("Missing `enabled` field")]
    NoField,
    #[error("Target `enabled` field must be of type `bool`")]
    FieldNotBool,
    #[error("Could not automatically locate field. Please add the `#[enabled]` attribute to a boolean field.")]
    MissingEnabledAttr,
    #[error("Multiple `#[enabled]` attributes found. Please remove all but one.")]
    MultipleEnabledAttrs,
}