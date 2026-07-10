use proc_macro::TokenStream;
use quote::quote;
use std::fmt::Display;
use syn::{DeriveInput, Fields, Type, parse_macro_input};

#[proc_macro_derive(DebugOption, attributes(enabled))]
pub fn derive_debug_option(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let syn::Data::Struct(ref data) = input.data else {
        return TokenStream::from(
            syn::Error::new(input.ident.span(), DebugOptionDeriveError::NonStruct)
                .to_compile_error(),
        );
    };

    let field_access = match data.fields {
        Fields::Named(ref fields) => {
            // Rule 3: Look for a bool field called "enabled"
            if let Some(field) = fields
                .named
                .iter()
                .find(|f| f.ident.as_ref().unwrap() == "enabled")
            {
                if !is_bool(&field.ty) {
                    return compile_error(field, DebugOptionDeriveError::FieldNotBool);
                }
                let ident = field.ident.as_ref().unwrap();
                quote!(#ident)
            } else {
                // Rule 4: Require #[enabled] attribute
                let enabled_fields: Vec<_> = fields
                    .named
                    .iter()
                    .filter(|f| has_enabled_attr(f))
                    .collect();

                match enabled_fields.len() {
                    0 => {
                        return compile_error(
                            &input.ident,
                            DebugOptionDeriveError::MissingEnabledAttr,
                        );
                    }
                    1 => {
                        let field = enabled_fields[0];
                        if !is_bool(&field.ty) {
                            return compile_error(field, DebugOptionDeriveError::FieldNotBool);
                        }
                        let ident = field.ident.as_ref().unwrap();
                        quote!(#ident)
                    }
                    _ => {
                        return compile_error(
                            &input.ident,
                            DebugOptionDeriveError::MultipleEnabledAttrs,
                        );
                    }
                }
            }
        }
        Fields::Unnamed(ref fields) => {
            // Rule 1: Tuple struct with only one field
            if fields.unnamed.len() == 1 {
                let field = fields.unnamed.first().unwrap();
                if !is_bool(&field.ty) {
                    return compile_error(field, DebugOptionDeriveError::FieldNotBool);
                }
                quote!(0)
            } else {
                // Rule 2: Tuple struct with multiple fields, require #[enabled]
                let enabled_fields: Vec<_> = fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .filter(|(_, f)| has_enabled_attr(f))
                    .collect();

                match enabled_fields.len() {
                    0 => {
                        return compile_error(
                            &input.ident,
                            DebugOptionDeriveError::MissingEnabledAttr,
                        );
                    }
                    1 => {
                        let (idx, field) = enabled_fields[0];
                        if !is_bool(&field.ty) {
                            return compile_error(field, DebugOptionDeriveError::FieldNotBool);
                        }
                        let index = syn::Index::from(idx);
                        quote!(#index)
                    }
                    _ => {
                        return compile_error(
                            &input.ident,
                            DebugOptionDeriveError::MultipleEnabledAttrs,
                        );
                    }
                }
            }
        }
        Fields::Unit => {
            return TokenStream::from(
                syn::Error::new(input.ident.span(), DebugOptionDeriveError::NoField)
                    .to_compile_error(),
            );
        }
    };

    // Grab generics so we can implement the trait for generic structs too
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics DebugOption for #name #ty_generics #where_clause {
            fn get(&self) -> bool {
                self.#field_access
            }

            fn set(&mut self, value: bool) {
                self.#field_access = value;
            }
        }
    };

    TokenStream::from(expanded)
}

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

fn compile_error<T: quote::ToTokens>(tokens: T, error: DebugOptionDeriveError) -> TokenStream {
    TokenStream::from(syn::Error::new_spanned(tokens, error).to_compile_error())
}

enum DebugOptionDeriveError {
    NonStruct,
    NoField,
    FieldNotBool,
    MissingEnabledAttr,
    MultipleEnabledAttrs,
}

impl Display for DebugOptionDeriveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DebugOptionDeriveError::NonStruct => {
                write!(f, "DebugOption can only be derived for structs")
            }
            DebugOptionDeriveError::NoField => write!(f, "Missing `enabled` field"),
            DebugOptionDeriveError::FieldNotBool => {
                write!(f, "Target `enabled` field must be of type `bool`")
            }
            DebugOptionDeriveError::MissingEnabledAttr => write!(
                f,
                "Could not automatically locate field. Please add the `#[enabled]` attribute to a boolean field."
            ),
            DebugOptionDeriveError::MultipleEnabledAttrs => write!(
                f,
                "Found multiple `#[enabled]` attributes. Only one field can be used."
            ),
        }
    }
}
