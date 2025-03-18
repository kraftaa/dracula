#![allow(proc_macro_derive_resolution_fallback)]

table! {
    purchase_orders (id) {
        id -> Int4,
        version_number -> Int4,
        referenced_po_id -> Nullable<Int4>,
        ad_po_number -> Nullable<Varchar>,
        payment_terms -> Nullable<Int4>,
        po_number -> Nullable<Varchar>,
        po_created_at -> Nullable<Timestamp>,
        accepted -> Nullable<Bool>,
        rejected -> Nullable<Bool>,
        obsoleted -> Nullable<Bool>,
        completed -> Nullable<Bool>,
        justifications -> Array<Varchar>,
        commission_rate -> Nullable<Numeric>,
        provider_shipping_address_id -> Nullable<Int4>,
        customer_shipping_address_id -> Nullable<Int4>,
        cpo_id -> Nullable<Int4>,
        proposal_id -> Nullable<Int4>,
        created_at -> Nullable<Timestamp>,
        uuid -> Nullable<Uuid>,
        total_price -> Numeric,
        subtotal_price -> Numeric,
        provider_id -> Nullable<Int4>,
        order_id -> Nullable<Int4>,
        #[sql_name = "type"]
        type_ -> Nullable<Varchar>,
        supplier_updated -> Nullable<Bool>,
        updated_at -> Nullable<Timestamp>,
        user_id -> Nullable<Int4>,
        request_id -> Nullable<Int4>,
    }
}
