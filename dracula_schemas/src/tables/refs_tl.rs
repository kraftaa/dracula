#![allow(proc_macro_derive_resolution_fallback)]

table! {
    refs (id) {
        id -> Int8,
        # [sql_name = "type"]
        type_ -> Nullable<Varchar>,
        fields -> Nullable<Jsonb>,
        reference_of_type -> Nullable<Varchar>,
        reference_to_type -> Nullable<Varchar>,
        reference_of_id -> Nullable<Int4>,
        reference_to_id -> Nullable<Int4>,
        uuid -> Uuid,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}
