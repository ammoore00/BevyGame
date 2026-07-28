use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Attribute, Fields, GenericArgument, ItemStruct, PathArguments, Type, parse_macro_input,
    parse_quote,
};

#[proc_macro_attribute]
pub fn maybe_fields(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item = parse_macro_input!(input as ItemStruct);

    let Fields::Named(fields) = &mut item.fields else {
        return syn::Error::new_spanned(
            &item,
            "#[maybe_fields] only supports structs with named fields",
        )
        .to_compile_error()
        .into();
    };

    for field in &mut fields.named {
        if is_maybe_type(&field.ty) {
            if !has_serde_default(field.attrs.as_slice()) {
                field.attrs.push(parse_quote! {
                    #[serde(default)]
                });
            }

            if !has_serde_skip_serializing_if(field.attrs.as_slice()) {
                field.attrs.push(parse_quote! {
                    #[serde(skip_serializing_if = "Option::is_none")]
                });
            }
        }
    }

    quote!(#item).into()
}

fn is_maybe_type(ty: &Type) -> bool {
    let Type::Path(type_path) = ty else {
        return false;
    };

    let Some(segment) = type_path.path.segments.last() else {
        return false;
    };

    if segment.ident != "Maybe" {
        return false;
    }

    matches!(
        &segment.arguments,
        PathArguments::AngleBracketed(args)
            if args.args.iter().any(|arg| matches!(arg, GenericArgument::Type(_)))
    )
}

fn has_serde_default(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("serde") && attr.meta.to_token_stream().to_string().contains("default")
    })
}

fn has_serde_skip_serializing_if(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("serde")
            && attr
                .meta
                .to_token_stream()
                .to_string()
                .contains("skip_serializing_if")
    })
}
