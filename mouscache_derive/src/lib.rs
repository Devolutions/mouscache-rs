#![recursion_limit = "128"]
extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate syn;

use syn::{Ident, DeriveInput, Attribute};
use syn::Meta::{List, NameValue, Word};
use syn::NestedMeta::{Literal, Meta};
use quote::Tokens;
use proc_macro2::Span;
use std::str::FromStr;

struct DataAttribute {
    expires: Option<usize>,
    rename: Option<String>,
}

#[allow(dead_code)]
struct FieldAttribute {
    skip: bool,
    rename: Option<String>,
    key: bool,
}

#[proc_macro_derive(Cacheable, attributes(cache))]
pub fn derive_cacheable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();

    match impl_cacheable(&input) {
        Ok(gen) => gen.into(),
        Err(msg) => panic!(msg),
    }
}

fn value_from_lit<T: FromStr>(lit: &syn::Lit, attr_name: &str) -> Result<T, String> {
    if let &syn::Lit::Str(ref lit) = lit {
        let value = T::from_str(&lit.value()).map_err(move |_| { format!("Unable to parse attribute value for {}", attr_name) });
        return value;
    } else {
        Err("Unable to parse attribute value".to_string())
    }
}

fn get_meta_items(attr: &syn::Attribute) -> Option<Vec<syn::NestedMeta>> {
    if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "cache" {
        match attr.interpret_meta() {
            Some(List(ref meta)) => Some(meta.nested.iter().cloned().collect()),
            _ => {
                // TODO: produce an error
                None
            }
        }
    } else {
        None
    }
}

fn validate_data_attributes(attrs: &Vec<Attribute>) -> Result<DataAttribute, String> {
    let mut expires: Option<usize> = None;
    let mut rename: Option<String> = None;
    for meta_items in attrs.iter().filter_map(get_meta_items) {
        for meta in meta_items {
            match meta {
                Meta(NameValue(ref m)) if m.ident == "expires" => {
                    let expiration_time: usize = value_from_lit(&m.lit, "expires")?;
                    expires = Some(expiration_time);
                }
                Meta(NameValue(ref m)) if m.ident == "rename" => {
                    let name: String = value_from_lit(&m.lit, "rename")?;
                    rename = Some(name);
                }
                Meta(List(ref _m)) => return Err("There is no list attribute you can use on data types with mouscache".to_string()),
                Meta(Word(_name)) => return Err("There is no word attribute you can use on data types with mouscache".to_string()),
                Literal(_) => return Err("There is no litteral attribute you can use on data types with mouscache".to_string()),
                _ => return Err("Invalid mouscache attributes".to_string()),
            }
        }
    }

    Ok(DataAttribute {
        expires,
        rename,
    })
}

#[allow(dead_code)]
fn validate_fields_attributes(_attrs: Vec<Attribute>) -> Result<FieldAttribute, String> {
    Err("".to_string())
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

    let data_attrs = validate_data_attributes(&input.attrs)?;

    let base_func = expand_base_function(ident, data_attrs);

    let redis_func = expand_redis_function(input)?;

    Ok(quote! {
        impl ::mouscache::Cacheable for #ident {
            #base_func

            #redis_func
        }
    })
}

fn expand_base_function(ident: &Ident, data_attrs: DataAttribute) -> Tokens {
    let ident = if let Some(name) = data_attrs.rename {
        Ident::new(name.as_str(), ident.span())
    } else {
        ident.clone()
    };

    let expires_after_func = if let Some(ttl) = data_attrs.expires {
        quote! {
            fn expires_after(&self) -> Option<usize> {
                Option::from(#ttl)
            }
        }
    } else {
        quote! {
            fn expires_after(&self) -> Option<usize> {
                None
            }
        }
    };

    quote! {
        #[inline]
        fn model_name() -> &'static str where Self: Sized {
            stringify!(#ident)
        }

        fn as_any(&self) -> &Any {
            self
        }

        #expires_after_func
    }
}

fn expand_redis_function(input: &DeriveInput) -> Result<Tokens, String> {
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
                        _ => return Err(CacheError::Other(format!("Unable to parse field {} from string into {}", stringify!(#ident), stringify!(#(f.ty))))),
                    }
                } else {
                   return Err(CacheError::Other(format!("Unable to parse field {}", stringify!(#ident))));
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
            return Err(CacheError::Other(String::new()));
        }
    })
}