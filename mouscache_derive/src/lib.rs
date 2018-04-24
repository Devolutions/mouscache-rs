//! This crate provides Mouscache's derive macros.
//!
//! ```rust
//! #[macro_use]
//! extern crate mouscache_derive;
//!
//! #[derive(Cacheable)]
//! struct S;
//!
//! fn main() {}
//! ```
//!
//! Please refer to [mouscache-rs] for how to set this up.
//!
//! [mouscache-rs]: https://github.com/wayk/mouscache-rs


#![recursion_limit = "128"]
extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate syn;

mod attr;
mod derive;

#[proc_macro_derive(Cacheable, attributes(cache))]
pub fn derive_cacheable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match derive::impl_cacheable(input) {
        Ok(gen) => gen.into(),
        Err(msg) => panic!(msg),
    }
}