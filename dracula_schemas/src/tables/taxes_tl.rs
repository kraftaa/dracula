#![allow(proc_macro_derive_resolution_fallback)]

table! {
    taxes (id) {
        id -> Int4,
        taxable_id -> Nullable<Int4>,
        taxable_type -> Nullable<Varchar>,
        amount -> Numeric,
        category -> Nullable<Varchar>,
        currency -> Nullable<Varchar>,
        description -> Nullable<Text>,
        locale -> Nullable<Varchar>,
        notes -> Nullable<Text>,
        percent_rate -> Numeric,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
        #[sql_name = "type"]
        type_ -> Nullable<Varchar>,
    }
}
