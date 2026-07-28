mod debug_option;

use proc_macro::TokenStream;
use std::error::Error;

#[proc_macro_derive(DebugOption, attributes(enabled))]
pub fn derive_debug_option(input: TokenStream) -> TokenStream {
    debug_option::derive(input)
}

fn compile_error_spanned<T: quote::ToTokens>(tokens: T, error: impl Error) -> TokenStream {
    TokenStream::from(syn::Error::new_spanned(tokens, error).to_compile_error())
}

fn compile_error(span: proc_macro2::Span, error: impl Error) -> TokenStream {
    TokenStream::from(syn::Error::new(span, error).to_compile_error())
}
