use proc_macro::TokenStream;
use quote::quote;

mod lerp;

#[proc_macro_derive(Lerp)]
pub fn derive_lerp(input: TokenStream) -> TokenStream {
    lerp::derive_lerp_inner(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn the_crate() -> proc_macro2::TokenStream {
    quote! {crate}
}

enum Fields {
    Named(Vec<syn::Ident>),
    Unnamed(Vec<syn::Member>),
}

fn parse_input(input: TokenStream) -> syn::Result<(syn::Ident, Fields)> {
    let input: syn::DeriveInput = syn::parse(input)?;
    let name = input.ident;
    Ok(match input.data {
        syn::Data::Struct(d) => match d.fields {
            syn::Fields::Named(f) => (
                name,
                Fields::Named(
                    f.named
                        .iter()
                        .map(|f| f.ident.clone().unwrap())
                        .collect::<Vec<_>>(),
                ),
            ),
            syn::Fields::Unnamed(f) => (
                name,
                Fields::Unnamed(
                    (0..f.unnamed.len())
                        .map(|i| {
                            syn::Member::Unnamed(syn::Index {
                                index: i as u32,
                                span: proc_macro2::Span::call_site(),
                            })
                        })
                        .collect(),
                ),
            ),
            _ => panic!("struct must contain data"),
        },
        _ => panic!("only structs are supported"),
    })
}
