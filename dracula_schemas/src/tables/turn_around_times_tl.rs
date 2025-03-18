#![allow(proc_macro_derive_resolution_fallback)]

table! {
    turn_around_times (id) {
        id -> Int4,
        turnaroundable_id -> Nullable<Int4>,
        turnaroundable_type -> Nullable<Varchar>,
        min -> Nullable<Int8>,
        max -> Nullable<Int8>,
        display_units -> Nullable<Varchar>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}
