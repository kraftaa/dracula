#![allow(proc_macro_derive_resolution_fallback)]

table! {
requests (id) {
    id  -> Int4,
    name -> Nullable<Varchar>,
    description -> Nullable<Text>,
    quantity -> Int4,
    request_type -> Nullable<Varchar>,
    reason_for_cancelling -> Nullable<Text>,
    filters -> Nullable<Array<Nullable<Varchar>>>, // Nullable<Vec<Varchar>>
    closed -> Bool,
    cancelled -> Bool,
    on_hold -> Bool,
    status -> Nullable<Varchar>,
    exclude -> Bool,
    ordered_at -> Nullable<Timestamp>,
    user_id -> Nullable<Int4>,
    ware_id -> Nullable<Int4>,
    organization_id -> Nullable<Int4>,
    uuid -> Uuid,
    created_at -> Nullable<Timestamp>,
    updated_at  -> Nullable<Timestamp>,
    }
}
