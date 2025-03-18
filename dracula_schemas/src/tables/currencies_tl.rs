table! {
    currencies (id) {
        id -> Int4,
        exchangable_id -> Nullable<Int4>,
        exchangable_type -> Nullable<Varchar>,
        currency -> Nullable<Varchar>,
        conversion_rate -> Nullable<Numeric>, //default = 1.00
        conversion_set_at -> Nullable<Timestamp>,
        conversion_history -> Nullable<Jsonb>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}
