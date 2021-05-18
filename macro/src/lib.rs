use proc_macro::{TokenStream};

use proc_macro2::Span;
use quote::quote;
use syn::braced;
use syn::Ident;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::Result;
use syn::token::{Brace};
use syn::{Type};
use syn::Token;


struct Syntax  {
    ident: Ident,
    _brace_token: Brace,
    brace_fields: Punctuated<Type, Token![,]>,
}

struct Field {
    name: Ident,
    ty: Type
}

struct PoolStructure {
    name: Ident,
    fields: Vec<Field>
}


impl Parse for PoolStructure {

    fn parse(stream: ParseStream) -> Result<Self> {
        let content;
        let syntax = Syntax {
            ident: stream.parse().unwrap(),
            _brace_token: braced!(content in stream),
            brace_fields: content.parse_terminated(Type::parse).unwrap()
        };

        let type_vec: Vec<Type> = syntax.brace_fields.into_iter().collect();
        let fields: Vec<Field> = type_vec.into_iter()
            .enumerate()
            .map(|(i, t)|
                Field {
                    name: Ident::new(format!("_com_{}_", i).as_str(), Span::call_site()),
                    ty: t
                }
            )
            .collect();
        Ok(PoolStructure{
            name: syntax.ident,
            fields
        })
    }
}


#[proc_macro]
pub fn pool(input: TokenStream) -> TokenStream {
    let pool_structure = syn::parse_macro_input!(input as PoolStructure);
    let pool_name = pool_structure.name;
    let field_names: Vec<Ident> = pool_structure.fields.iter().map(|f| f.name.clone()).collect();
    let field_types: Vec<Type> = pool_structure.fields.iter().map(|f| f.ty.clone()).collect();

    let struct_def = quote!{
        #[derive(Default)]
        pub struct #pool_name {
            #(#field_names: std::collections::HashMap<String, async_std::sync::Arc<#field_types>>),*
        }
    };

    let comp_trait_impl = quote!{
        #(
            impl chord_common::component::HasComponent<#field_types> for #pool_name {
                fn add(&mut self, name: &str, component: async_std::sync::Arc<#field_types>) {
                    self.#field_names.insert(name.to_owned(), component);
                }

                fn get(&self, name: &str) -> Option<async_std::sync::Arc<#field_types>> {
                    self.#field_names.get(name).map(|c| c.clone())
                }

               fn get_all(&self) -> Vec<(&str, async_std::sync::Arc<#field_types>)> {
                    self.#field_names.iter().map(|(k,v)| (k.as_str(), v.clone())).collect()
                }
            }
        )*
    };

    let pool_static_var = Ident::new(format!("__pool_{}__", pool_name).as_str(), Span::call_site());

    let pool_def = quote!{
        static mut #pool_static_var: Option<#pool_name> = Option::None;
        impl #pool_name {

            fn pool_init() -> &'static mut #pool_name{
                unsafe {
                    #pool_static_var = Some(#pool_name::default());
                    #pool_static_var.as_mut().unwrap()
                }
            }

            pub fn pool_ref() -> &'static #pool_name {
                unsafe { #pool_static_var.as_ref().unwrap() }
            }
        }
    };


    let tokens = quote! {
        #struct_def
        #comp_trait_impl
        #pool_def
    };

    TokenStream::from(tokens)
}
