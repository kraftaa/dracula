table! {
    inventory_groups (id) {
        id -> Int4,
        index -> Int4,
        metadata -> Nullable<Jsonb>,
        shipping_cost -> Numeric,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
        uuid -> Nullable<Uuid>,
        deleted_at -> Nullable<Timestamp>,
    }
}