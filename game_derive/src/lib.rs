extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{Data, DeriveInput, Type};

#[proc_macro_derive(Persisted, attributes(group))]
pub fn persisted_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    generate_persisted_trait(&ast).into()
}

#[proc_macro_derive(Domain)]
pub fn domain_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    generate_domain_trait(&ast).into()
}

#[proc_macro_derive(Id)]
pub fn id_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    generate_id_trait(&ast).into()
}

const SQL_TYPES: [&'static str; 14] = [
    "String", "bool", "i8", "i16", "i32", "i64", "isize", "u8", "u16", "u32", "f32", "f64", "u64",
    "usize",
];

fn generate_persisted_trait(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let code = match &ast.data {
        Data::Enum(_) => panic!("Persist trait for enum not implemented yet"),
        Data::Struct(data) => {
            let mut binders = vec![];
            let mut columns = vec![];
            let mut mapping = vec![];
            let mut kind = None;
            let mut group = "undefined!".to_string();
            for (index, field) in data.fields.iter().enumerate() {
                let field_ident = field.ident.as_ref().unwrap();
                let field_name = format!("{}", field_ident);

                if field_name == "kind" {
                    let kind_type = match &field.ty {
                        Type::Path(path) => path.to_token_stream(),
                        _ => quote! { () },
                    };
                    kind = Some(kind_type);
                    continue;
                }

                if field
                    .attrs
                    .iter()
                    .map(|a| a.path.to_token_stream().to_string())
                    .filter(|attribute| attribute == "group")
                    .last()
                    .is_some()
                {
                    group = field_name.clone();
                }

                let index = index + 2; // 1-based, 1-reserved
                let is_sql_type = match &field.ty {
                    Type::Path(path) => {
                        let name = path.to_token_stream().to_string();
                        SQL_TYPES.contains(&&*name) || name.ends_with("Id")
                    }
                    _ => false,
                };

                let bind_value = if is_sql_type {
                    quote! { &self.#field_ident }
                } else {
                    quote! { crate::persistence::to_json_value(&self.#field_ident) }
                };

                let map_value = if is_sql_type {
                    quote! { row.get(#field_name)? }
                } else {
                    quote! { crate::persistence::parse_json_value(row.get(#field_name)?) }
                };

                binders.push(quote! {
                    statement.raw_bind_parameter(#index, #bind_value)?;
                });
                columns.push(quote! {
                    #field_name.to_string()
                });
                mapping.push(quote! {
                    #field_ident: #map_value,
                })
            }

            let parse = if kind.is_some() {
                quote! {
                    fn parse_known(row: &rusqlite::Row, kind: Self::Kind) -> Result<Self, rusqlite::Error> {
                        Ok(Self {
                            kind,
                            #(#mapping)*
                        })
                    }
                }
            } else {
                quote! {
                    fn parse(row: &rusqlite::Row) -> Result<Self, rusqlite::Error> {
                        Ok(Self {
                            #(#mapping)*
                        })
                    }
                }
            };

            let kind = kind.unwrap_or(quote! { () });
            quote! {
                impl crate::persistence::Persist for #name {
                    type Kind = #kind;

                    fn entry_id(&self) -> usize {
                        self.id.into()
                    }

                    fn columns() -> Vec<String> {
                        vec![#(#columns),*]
                    }

                    fn group() -> String {
                        #group.to_string()
                    }

                    fn bind(&self, statement: &mut rusqlite::Statement) -> rusqlite::Result<()> {
                        #(#binders)*
                        Ok(())
                    }

                    #parse
                }
            }
        }
        Data::Union(_) => panic!("Persist trait for union not implemented yet"),
    };
    code.into()
}

fn generate_domain_trait(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let code = match &ast.data {
        Data::Enum(_) => panic!("Persist trait for enum not implemented yet"),
        Data::Struct(data) => {
            let mut loaders = vec![];
            let mut dumpers = vec![];
            for field in data.fields.iter() {
                let field_ident = field.ident.as_ref().unwrap();
                match &field.ty {
                    Type::Path(path) => {
                        let ty = path.to_token_stream().to_string();
                        if ty.starts_with("Readonly <") {
                            loaders.push(quote! {
                                self.#field_ident.update(connection);
                            });
                        }
                        if ty.starts_with("Mutable <") {
                            loaders.push(quote! {
                                self.#field_ident.load(connection);
                            });
                            dumpers.push(quote! {
                                self.#field_ident.dump(connection);
                            });
                        }
                    }
                    _ => {}
                }
            }
            quote! {
                impl #name {
                    pub fn load(&mut self, connection: &rusqlite::Connection) {
                        #(#loaders)*
                    }

                    pub fn dump(&mut self, connection: &rusqlite::Connection) {
                        #(#dumpers)*
                    }
                }
            }
        }
        Data::Union(_) => panic!("Persist trait for union not implemented yet"),
    };
    code.into()
}

fn generate_id_trait(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let code = match &ast.data {
        Data::Enum(_) => panic!("Persist trait for enum not implemented yet"),
        Data::Struct(_) => {
            quote! {

                impl rusqlite::types::FromSql for #name {
                    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
                        usize::column_result(value).map(|value| #name(value))
                    }
                }

                impl rusqlite::ToSql for #name {
                    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
                        self.0.to_sql()
                    }
                }

                impl Into<usize> for #name {
                    fn into(self) -> usize {
                        self.0
                    }
                }

                impl From<usize> for #name {
                    fn from(value: usize) -> Self {
                        #name(value)
                    }
                }
            }
        }
        Data::Union(_) => panic!("Persist trait for union not implemented yet"),
    };
    code.into()
}
