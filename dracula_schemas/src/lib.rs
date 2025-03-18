#![allow(unknown_lints)]
#![allow(incomplete_features)]
#![recursion_limit = "512"]
#![allow(proc_macro_derive_resolution_fallback)]
#[macro_use]
extern crate diesel;

pub mod models;
pub use models::*;

pub mod tables;
pub use tables::*;
