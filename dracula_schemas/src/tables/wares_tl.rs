#![allow(proc_macro_derive_resolution_fallback)]

table! {
    wares (id) {
        id -> Int4,
        name -> Nullable < Varchar >,
        created_at -> Nullable < Timestamp >,
        updated_at -> Nullable < Timestamp >,
        slug -> Nullable<Varchar>,
        description -> Nullable< Varchar >,
        ware_type -> Nullable< Varchar >,
    }
}
