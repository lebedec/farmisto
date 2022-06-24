extern crate proc_macro;

use proc_macro::TokenStream;

use quote::quote;
use syn::{Data, DeriveInput};

#[proc_macro_derive(Persist)]
pub fn persisted_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    generate_trait(&ast).into()
}

fn generate_trait(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let code = match &ast.data {
        Data::Enum(_data) => panic!("Persist trait for enum not implemented yet"),
        Data::Struct(_data) => {
            quote! {
                impl #name {
                    pub fn persist(&self, entry: i64) {
                        info!("persist in {}", entry)
                    }
                }
            }
        }
        Data::Union(_) => panic!("Persist trait for union not implemented yet"),
    };
    code.into()
}
