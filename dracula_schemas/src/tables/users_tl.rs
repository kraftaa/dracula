#![allow(proc_macro_derive_resolution_fallback)]

table! {
    users (id) {
        id -> Int4,
        shipping_address_id -> Nullable<Int4>,
        billing_address_id -> Nullable<Int4>,
        active -> Bool,
        company -> Nullable<Varchar>,
        email -> Nullable<Varchar>,
        first_name -> Nullable<Varchar>,
        last_name -> Nullable<Varchar>,
        last_request_at -> Nullable<Timestamp>,
        last_sign_in_at -> Nullable<Timestamp>,
        created_at -> Nullable<Timestamp>,
        sso_attributes -> Nullable<Jsonb>,
        user_agreement_signed_at -> Nullable<Timestamp>,
        expired_at -> Nullable<Timestamp>,
        uuid -> Nullable<Uuid>,
    }
}
