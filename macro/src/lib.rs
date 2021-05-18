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

        let ident: Ident = stream.parse().unwrap();

        let content;
        let _brace_token: Brace = braced!(content in stream);
        println!("content: {}", content);

        let brace_fields: Punctuated<Type, Token![,]> = content.parse_terminated(Type::parse).unwrap();

        let syntax = Syntax {
            ident,
            _brace_token,
            brace_fields,
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
            #(#field_names: Option<async_std::sync::Arc<#field_types>>),*
        }
    };

    let comp_trait_impl = quote!{
        #(
            impl chord_common::component::HasComponent<#field_types> for #pool_name {
                fn set(&mut self, component: async_std::sync::Arc<#field_types>) {
                    self.#field_names = Some(component)
                }

                fn get(&self) -> Option<async_std::sync::Arc<#field_types>> {
                    self.#field_names.as_ref().map(|c| c.clone())
                }
            }
        )*
    };


    let pool_def = quote!{
        static mut POOL: Option<#pool_name> = Option::None;
        impl #pool_name {

            fn pool_init() -> &'static mut #pool_name{
                unsafe {
                    POOL = Some(#pool_name::default());
                    POOL.as_mut().unwrap()
                }
            }

            pub fn pool_ref() -> &'static #pool_name {
                unsafe { POOL.as_ref().unwrap() }
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
