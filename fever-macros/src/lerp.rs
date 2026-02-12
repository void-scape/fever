use proc_macro::TokenStream;
use quote::quote;

pub fn derive_lerp_inner(input: TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let (name, fields) = crate::parse_input(input)?;
    let the_crate = crate::the_crate();
    Ok(match fields {
        crate::Fields::Named(fields) => {
            quote! {
                impl #the_crate::animation::Lerp for #name {
                    fn lerp(&self, rhs: &Self, t: f32) -> Self {
                        Self {
                            #(#fields: <_ as #the_crate::animation::Lerp>::lerp(&self.#fields, &rhs.#fields, t)),*
                        }
                    }
                    fn add(&self, rhs: &Self) -> Self {
                        Self {
                            #(#fields: self.#fields.add(&rhs.#fields)),*
                        }
                    }
                }
            }
        }
        crate::Fields::Unnamed(fields) => {
            quote! {
                impl #the_crate::animation::Lerp for #name {
                    fn lerp(&self, rhs: &Self, t: f32) -> Self {
                        Self(#(<_ as #the_crate::animation::Lerp>::lerp(&self.#fields, &rhs.#fields, t)),*)
                    }
                    fn add(&self, rhs: &Self) -> Self {
                        Self(#(self.#fields.add(&rhs.#fields)),*)
                    }
                }
            }
        }
    })
}
