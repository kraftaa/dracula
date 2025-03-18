#![allow(proc_macro_derive_resolution_fallback)]

table! {
  orders (id) {
    id -> Int4,
    provider_id  -> Nullable<Int4>,
    request_id  -> Nullable<Int4>,
    client_id  -> Nullable<Int4>,
    orgaization_id  -> Nullable<Int4>,
    obligations -> Nullable<Jsonb>,
    state -> Varchar,
    declined -> Bool,
    on_hold -> Bool,
    order_cancelled -> Bool,
    request_cancelled -> Bool,
    last_computed_status -> Nullable<Varchar>,
    uuid -> Uuid,
    created_at -> Nullable<Timestamp>,
    updated_at -> Nullable<Timestamp>,
    booster -> Nullable<Double>,
  }
}
