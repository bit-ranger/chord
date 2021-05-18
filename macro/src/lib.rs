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
use std::hash::Hash;
use std::collections::HashSet;


struct Syntax  {
    ident: Ident,
    _brace_token: Brace,
    brace_fields: Punctuated<Type, Token![,]>,
}

struct Structure {
    name: Ident,
    tys: Vec<Type>
}


impl Parse for Structure {

    fn parse(stream: ParseStream) -> Result<Self> {
        let content;
        let syntax = Syntax {
            ident: stream.parse().unwrap(),
            _brace_token: braced!(content in stream),
            brace_fields: content.parse_terminated(Type::parse).unwrap()
        };

        let type_vec: Vec<Type> = syntax.brace_fields.into_iter().collect();

        Ok(Structure {
            name: syntax.ident,
            tys: type_vec
        })
    }
}

fn has_unique_elements<T>(iter: T) -> bool
    where
        T: IntoIterator,
        T::Item: Eq + Hash,
{
    let mut unique = HashSet::new();
    iter.into_iter().all(move |x| unique.insert(x))
}

#[proc_macro]
pub fn container(input: TokenStream) -> TokenStream {
    let container_structure = syn::parse_macro_input!(input as Structure);
    let container_name = container_structure.name;

    if !has_unique_elements(container_structure.tys.clone()) {
        panic!("duplicate type exist")
    }

    let field_types: Vec<Type> = container_structure.tys.clone();
    let field_names: Vec<Ident> = field_types
        .clone()
        .into_iter()
        .enumerate()
        .map(|(i, _t)| Ident::new(format!("_com_{}_", i).as_str(), Span::call_site()))
        .collect();

    let struct_def = quote!{
        #[derive(Default)]
        pub struct #container_name {
            #(#field_names: std::collections::HashMap<String, async_std::sync::Arc<#field_types>>),*
        }
    };

    let comp_trait_impl = quote!{
        #(
            impl chord_common::component::HasComponent<#field_types> for #container_name {
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

    let static_var = Ident::new(format!("__CONTAINER_{}__", container_name).as_str(), Span::call_site());

    let container_def = quote!{
        static mut #static_var: Option<#container_name> = Option::None;
        impl #container_name {

            fn init() -> &'static mut #container_name{
                unsafe {
                    #static_var = Some(#container_name::default());
                    #static_var.as_mut().unwrap()
                }
            }

            pub fn borrow() -> &'static #container_name {
                unsafe { #static_var.as_ref().unwrap() }
            }
        }
    };


    let tokens = quote! {
        #struct_def
        #comp_trait_impl
        #container_def
    };

    TokenStream::from(tokens)
}
