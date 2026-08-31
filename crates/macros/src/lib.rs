mod command;

use proc_macro::TokenStream;
use syn::parse_macro_input;

#[proc_macro_derive(Args, attributes(arg))]
pub fn derive_args(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    match command::expand_args(input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.into_compile_error().into(),
    }
}
