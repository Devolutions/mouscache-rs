use syn;
use syn::{Attribute};
use syn::Meta::{List, NameValue, Word};
use syn::NestedMeta::{Literal, Meta};
use std::str::FromStr;

pub struct DataAttribute {
    pub expires: Option<usize>,
    pub rename: Option<String>,
}

#[allow(dead_code)]
pub struct FieldAttribute {
    pub skip: bool,
    pub rename: Option<String>,
    pub key: bool,
}

pub fn value_from_lit<T: FromStr>(lit: &syn::Lit, attr_name: &str) -> Result<T, String> {
    if let &syn::Lit::Str(ref lit) = lit {
        let value = T::from_str(&lit.value()).map_err(move |_| { format!("Unable to parse attribute value for {}", attr_name) });
        return value;
    } else {
        Err("Unable to parse attribute value".to_string())
    }
}

pub fn get_meta_items(attr: &syn::Attribute) -> Option<Vec<syn::NestedMeta>> {
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

pub fn validate_data_attributes(attrs: &Vec<Attribute>) -> Result<DataAttribute, String> {
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
pub fn validate_fields_attributes(_attrs: Vec<Attribute>) -> Result<FieldAttribute, String> {
    Err("".to_string())
}