#![allow(proc_macro_derive_resolution_fallback)]

table! {
    organizations (id) {
         id -> Int4,
         name -> Nullable<Varchar>,
         uuid -> Nullable<Uuid>,
         created_at -> Nullable<Timestamp>,
         updated_at -> Nullable<Timestamp>,
         parent_id -> Nullable<Int4>,
         domain -> Nullable<Varchar>,
         webstore -> Bool,
         archived_at -> Nullable<Timestamp>,
    }
}
