// TODO: currency is a method
// TODO: payment_terms is an alias for the purchase_order payment_terms
// TODO: customer_shipping_address is a table

#![allow(proc_macro_derive_resolution_fallback)]

table! {
    invoices (id) {
        id -> Int4,
        invoice_number -> Nullable<Varchar>,
        cancelled -> Bool,
        issued_at -> Nullable<Timestamp>,
        document_type -> Varchar,
        commission_rate -> Nullable<Numeric>,
        purchase_order_id -> Nullable<Int4>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
        uuid -> Nullable<Uuid>,
        subtotal_price -> Numeric,
        total_price -> Numeric,
        discount_amount -> Numeric,
        provider_id -> Nullable<Int4>,
        order_id  -> Nullable<Int4>,
        approved_for_payment -> Nullable<Bool>,
        discount -> Nullable<Varchar>,
    }
}
