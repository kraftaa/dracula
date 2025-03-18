#![allow(proc_macro_derive_resolution_fallback)]

table! {
  timepoints (id) {
    id -> Int4,
    name  -> Nullable<Varchar>,
    date -> Nullable<Timestamp>,
    slug  -> Nullable<Varchar>,
    order_id  -> Nullable<Int4>,
    uuid -> Uuid,
    created_at -> Nullable<Timestamp>,
    updated_at -> Nullable<Timestamp>,
    }
}
