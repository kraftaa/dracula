table! {
    milestones (id) {
        id -> Int4,
        itemizable_id -> Nullable<Int4>,
        itemizable_type -> Nullable<Varchar>,
        unit_price -> Numeric,
        total_price -> Numeric,
        #[sql_name = "type"]
        type_ -> Varchar,
        name -> Nullable<Varchar>,
        quantity -> Numeric,
        state -> Varchar,
        tax_rate -> Numeric,
        currency -> Nullable<Varchar>,
        unit_of_measure -> Nullable<Varchar>,
        classifications -> Jsonb,
        line_number -> Nullable<Int4>,
        comments -> Nullable<Text>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
        status -> Nullable<Varchar>,
        provider_id -> Nullable<Int4>,
        cancelled_at -> Nullable<Timestamp>,
        shipped_at -> Nullable<Timestamp>,
        estimated_date -> Nullable<Timestamp>,
    }
}
