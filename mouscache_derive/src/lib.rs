#![recursion_limit="128"]
extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate syn;

use syn::{Ident, Index, Member, DeriveInput};
use syn::spanned::Spanned;
use quote::Tokens;
use proc_macro2::Span;
use proc_macro::TokenStream;

#[proc_macro_derive(Cacheable)]
pub fn derive_cacheable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();

    match impl_cacheable(&input) {
        Ok(gen) => gen.into(),
        Err(msg) => panic!(msg),
    }
}

fn impl_cacheable(input: &DeriveInput) -> Result<Tokens, String> {
    let name: &Ident = &input.ident;

    let usages = expand_usages();

    let impl_block = expand_cacheable_impl_block(input)?;

    let dummy_const = Ident::new(&format!("_IMPL_DESERIALIZE_FOR_{}", name), Span::call_site());

    Ok(quote! {
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications, unused_imports)]
        const #dummy_const: () = {
            #usages

            #impl_block
        };
    })
}

fn expand_usages() -> Tokens {
    quote! {
        use std::any::Any;
        use std::string::ToString;
        use std::collections::hash_map::HashMap;
        use ::mouscache::CacheError;
        use ::mouscache::Result;
    }
}

fn expand_cacheable_impl_block(input: &DeriveInput) -> Result<Tokens, String> {
    let ident: &Ident = &input.ident;

    let base_func = expand_base_function(ident);

    let redis_func = expand_redis_function(input)?;

    Ok(quote! {
        impl ::mouscache::Cacheable for #ident {
            #base_func

            #redis_func
        }
    })
}

fn expand_base_function(ident: &Ident) -> Tokens {
    quote! {
        #[inline]
        fn model_name() -> &'static str where Self: Sized {
            stringify!(#ident)
        }

        fn as_any(&self) -> &Any {
            self
        }
    }
}

fn expand_redis_function(input: &DeriveInput)  -> Result<Tokens, String> {
    let struct_ident: &Ident = &input.ident;

    let fields = match input.data {
        syn::Data::Struct(ref data) => data.fields.clone(),
        syn::Data::Enum(_) => return Err(String::from("#[derive(Cacheable)] only apply to structs at this time")),
        syn::Data::Union(_) => return Err(String::from("#[derive(Cacheable)] only apply to structs at this time")),
    };

    let named_fields = match fields {
        syn::Fields::Named(ref f_named) => f_named.named.clone(),
        syn::Fields::Unnamed(_) => return Err(String::from("Unnamed fields are not implemented at this time -> #[derive(Cacheable)] only apply to structs with named fields")),
        syn::Fields::Unit => return Err(String::from("Unit fields are not implemented at this time -> #[derive(Cacheable)] only apply to structs with named fields")),
    };

    let mut field_tokens: Vec<Tokens> = Vec::new();

    for f in named_fields.iter() {
        if let Some(ref ident) = f.ident {
            field_tokens.push(quote!(String::from(stringify!(#ident)), self.#ident.to_string()))
        }
    }

    let hmap_ident = Ident::new("map", Span::call_site());

    let mut field_deser_tokens: Vec<Tokens> = Vec::new();

    for f in named_fields.iter() {
        let t = f.ty.clone();
        if let Some(ref ident) = f.ident {
            field_deser_tokens.push(quote! {
                let #ident = if let Some(obj) = #hmap_ident.get(&stringify!(#ident).to_string()) {
                    match obj.parse::<#t>() {
                        Ok(o) => o,
                        _ => return Err(CacheError::RedisError(String::from(format!("Unable to parse field {} from string into {}", stringify!(#ident), stringify!(#(f.ty)))))),
                    }
                } else {
                   return Err(CacheError::RedisError(String::from(format!("Unable to parse field {}", stringify!(#ident)))));
                };
            });
        }
    }

    let mut struct_ident_vec: Vec<Tokens> = Vec::new();

    for f in named_fields.iter() {
        if let Some(ref ident) = f.ident {
            struct_ident_vec.push(quote!(#ident,));
        }
    }

    let return_token = quote! {
        return Ok(#struct_ident {
            #(#struct_ident_vec)*
        });
    };

    Ok(quote! {
        fn to_redis_obj(&self) -> Vec<(String, String)> {
            let mut temp_vec = Vec::new();
            #(temp_vec.push((#field_tokens));)*
            temp_vec
        }

        fn from_redis_obj(#hmap_ident: HashMap<String, String>) -> Result<Self> where Self: Sized {
            if #hmap_ident.len() > 0 {
                #(#field_deser_tokens)*

                #return_token
            }
            return Err(CacheError::NotConfigured);
        }
    })
}