#![allow(proc_macro_derive_resolution_fallback)]

table! {
    proposals (id) {
        id -> Int4,
        active -> Bool,
        description -> Text,
        obsoleted -> Bool,
        proposal_type -> Nullable<Varchar>,
        state -> Nullable<Varchar>,
        exclude -> Bool,
        justifications -> Nullable<Array<Varchar>>,
        commission_rate -> Nullable<Numeric>,
        user_full_name -> Nullable<Varchar>,
        user_id -> Nullable<Int4>,
        uuid -> Nullable<Uuid>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
        subtotal_price -> Numeric,
        total_price -> Numeric,
        provider_id -> Nullable<Int4>,
        tax_category -> Nullable<Varchar>,
        order_id -> Nullable<Int4>,
        #[sql_name = "type"]
        type_ -> Nullable<Varchar>,
        ware_id -> Nullable<Int4>,
        status -> Nullable<Varchar>,
    }
}
