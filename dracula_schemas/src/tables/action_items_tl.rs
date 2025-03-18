#![allow(proc_macro_derive_resolution_fallback)]

table! {
    action_items (id) {
        id -> Int8,
        action_performed -> Nullable<Varchar>,
        completed_at -> Nullable<Timestamp>,
        proposal_id -> Nullable<Int4>,
        purchase_order_id -> Nullable<Int4>,
        provider_id -> Nullable<Varchar>,
        uuid -> Nullable<Uuid>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
        order_id -> Nullable<Int4>,
        request_id -> Nullable<Int4>,
        note_id -> Nullable<Int4>,
        invoice_id -> Nullable<Int4>,
        request_name -> Nullable<Varchar>,
        organization_id -> Nullable<Int4>,
        #[sql_name = "type"]
        type_ -> Nullable<Varchar>,
        ads_ad_id -> Nullable<Int4>,
    }
}
