use proc_macro::TokenStream;
use syn::parse_macro_input;

mod derive;

#[proc_macro_derive(FieldDefs, attributes(field))]
pub fn derive_field_defs(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    match derive::expand(input) {
        Ok(expanded) => expanded.into(),
        Err(error) => error.to_compile_error().into(),
    }
}
