#![allow(proc_macro_derive_resolution_fallback)]

table! {
notes (id) {
    id -> Int8,
    title -> Nullable<Text>,
    body -> Nullable<Text>,
    status -> Nullable<Varchar>,
    request_id -> Nullable<Int4>,
    uuid -> Uuid,
    ware_id -> Nullable<Varchar>,
    user_id -> Nullable<Int4>,
    invoice_id -> Nullable<Int4>,
    purchase_order_id -> Nullable<Int4>,
    proposal_id -> Nullable<Int4>,
    created_at -> Nullable<Timestamp>,
    updated_at  -> Nullable<Timestamp>,
    }
}
