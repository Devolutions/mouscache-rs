#[cfg(test)]
extern crate mouscache;
#[cfg(test)]
#[macro_use]
extern crate mouscache_derive;

#[cfg(test)]
mod manual_impl_test;

#[cfg(test)]
mod derive_test;

#[cfg(test)]
mod concurrency_test;

#[cfg(test)]
mod redis_like_test;