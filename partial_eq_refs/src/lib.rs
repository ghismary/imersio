use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(PartialEqRefs)]
pub fn derive_partial_eq_refs(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_partial_eq_refs(&ast)
}

fn impl_partial_eq_refs(ast: &syn::DeriveInput) -> TokenStream {
    let ident = &ast.ident;
    let gen = quote! {
        impl PartialEq<&#ident> for #ident {
            fn eq(&self, other: &&#ident) -> bool {
                self == *other
            }
        }

        impl PartialEq<#ident> for &#ident {
            fn eq(&self, other: &#ident) -> bool {
                *self == other
            }
        }
    };
    gen.into()
}
