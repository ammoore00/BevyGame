use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{parse_macro_input, Expr, ItemStruct, LitBool, Meta, Token, Type, Lit, LitStr};

#[proc_macro_attribute]
pub fn resource_kind(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_struct = parse_macro_input!(item as ItemStruct);
    let struct_ident = &input_struct.ident;

    // 1. Parse attribute arguments: #[resource_kind(path = "...", asset_kind = ...)]
    let parser = Punctuated::<Meta, Token![,]>::parse_terminated;
    let metas = match parser.parse(attr) {
        Ok(metas) => metas,
        Err(err) => return err.to_compile_error().into(),
    };

    let mut path_expr: Option<LitStr> = None;
    let mut asset_kind: Option<Type> = None;
    let mut file_type: Option<Expr> = None;
    let mut visit_override: Option<LitBool> = None;

    for meta in metas {
        match meta {
            Meta::NameValue(nv) => {
                if nv.path.is_ident("path") {
                    let expr = nv.value;
                    match syn::parse2::<LitStr>(quote!(#expr)) {
                        Ok(lit) => path_expr = Some(lit),
                        Err(err) => return err.to_compile_error().into(),
                    }
                } else if nv.path.is_ident("asset_kind") || nv.path.is_ident("asset_type") {
                    let expr = nv.value;
                    match syn::parse2::<Type>(quote!(#expr)) {
                        Ok(ty) => asset_kind = Some(ty),
                        Err(err) => return err.to_compile_error().into(),
                    }
                } else if nv.path.is_ident("file_type") {
                    file_type = Some(nv.value);
                } else if nv.path.is_ident("visit_override") {
                    let expr = nv.value;
                    match syn::parse2::<LitBool>(quote!(#expr)) {
                        Ok(bl) => visit_override = Some(bl),
                        Err(err) => return err.to_compile_error().into(),
                    }
                } else {
                    return syn::Error::new(
                        nv.path.span(),
                        format!(
                            "Unknown argument `{}` in #[resource_kind(...)]",
                            quote!(#nv.path)
                        ),
                    )
                    .to_compile_error()
                    .into();
                }
            }
            _ => {
                return syn::Error::new(
                    meta.span(),
                    "#[resource_kind(...)] expects `key = value` pairs",
                )
                .to_compile_error()
                .into();
            }
        }
    }

    // Ensure required parameters are provided
    let path = match path_expr {
        Some(p) => {
            if file_type.is_none() {
                LitStr::new(&format!("data/{}", p.value()), p.span())
            } else {
                p
            }
        },
        None => {
            return syn::Error::new(
                input_struct.span(),
                "#[resource_kind(...)] attribute requires `path = \"...\"`",
            )
            .to_compile_error()
            .into();
        }
    };

    let asset_kind = match asset_kind {
        Some(ty) => ty,
        None => {
            return syn::Error::new(
                input_struct.span(),
                "#[resource_kind(...)] attribute requires `asset_kind = ...`",
            )
            .to_compile_error()
            .into();
        }
    };

    let file_type = file_type.unwrap_or_else(|| syn::parse_quote!(::data::resource::ResourceFileType::Data));

    let struct_name = struct_ident.to_string();
    let base_name = struct_name.strip_suffix("Resource").unwrap_or(&struct_name);
    let registry_ident = format_ident!("{}Registry", base_name);

    let visit_override = if let Some(visit_override) = visit_override && visit_override.value {
        quote! {
            fn visit(
                loc: ::data::loc::ResourceLocation<Self>,
                asset: Self::AssetKind,
                world: &mut ::bevy::prelude::World,
            ) -> Result<Self::AssetKind, ::data::resource::ResourceVisitError> {
                Self::visit(loc, asset, world)
            }
        }
    } else {
        quote! {
            fn visit(
                _loc: ::data::loc::ResourceLocation<Self>,
                asset: Self::AssetKind,
                _world: &mut ::bevy::prelude::World,
            ) -> Result<Self::AssetKind, ::data::resource::ResourceVisitError> {
                Ok(asset)
            }
        }
    };

    // 2. Output the original struct definition alongside the trait implementation

    let expanded = quote! {
        #[allow(unused)]
        #[derive(Hash, Eq, PartialEq, Debug, Clone, Copy, Default, Reflect)]
        #input_struct

        pub type #registry_ident = ::data::registry::ResourceRegistry<#struct_ident>;

        impl ::data::resource::ResourceKind for #struct_ident {
            type AssetKind = #asset_kind;

            const ROOT_DIR: &'static str = #path;
            const FILE_TYPE: ::data::resource::ResourceFileType = #file_type;

            #visit_override
        }
    };
    TokenStream::from(expanded)
}
