#![allow(unknown_lints)]
#![allow(incomplete_features)]
#![recursion_limit = "512"]
#![allow(proc_macro_derive_resolution_fallback)]

// Order matters!
extern crate openssl;
#[macro_use]
extern crate diesel;

extern crate log;
pub mod tasks;
pub use tasks::*;
