#![allow(proc_macro_derive_resolution_fallback)]

table! {
    ratings (id) {
        id -> Int4,
        value -> Int4,
        ware_id -> Nullable<Varchar>,
        spam -> Bool,
        provider_id  -> Nullable<Int4>,
        created_at -> Nullable<Timestamp>,
        ware_pg_id -> Nullable<Int4>,
        request_id -> Nullable<Int4>,
        order_id -> Nullable<Int4>,
        comment -> Nullable<Text>,
        organization_id  -> Nullable<Int4>,
        user_id -> Nullable<Int4>,
    }
}
