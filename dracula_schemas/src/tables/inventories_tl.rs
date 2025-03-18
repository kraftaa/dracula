table! {
    inventories (id) {
        id -> Int8,
        inventory_group_id -> Nullable<Int4>,
        data -> Nullable<Jsonb>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
        uuid -> Nullable<Uuid>,
        product_name ->  Nullable<Varchar>,
        product_url ->  Nullable<Varchar>,
        provider_id ->  Nullable<Int4>,
    }
}
