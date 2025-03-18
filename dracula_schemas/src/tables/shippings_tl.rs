#![allow(proc_macro_derive_resolution_fallback)]

table! {
    shippings (id) {
        id -> Int4,
        shipable_id -> Nullable<Int4>,
        shipable_type -> Nullable<Varchar>,
        cost -> Numeric,
        free_shipping -> Bool,
        currency -> Nullable<Varchar>,
        notes -> Nullable<Text>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}
