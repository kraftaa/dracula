#![allow(proc_macro_derive_resolution_fallback)]

table! {
    events (id) {
        id -> Int8,
        event -> Nullable<Varchar>,
        email -> Nullable<Varchar>,
        username -> Nullable<Varchar>,
        application -> Nullable<Varchar>,
        duration -> Nullable<Double>,
        remote_addr -> Nullable<Varchar>,
        host -> Nullable<Varchar>,
        action -> Nullable<Varchar>,
        controller -> Nullable<Varchar>,
        session_id -> Nullable<Varchar>,
        computer_id -> Nullable<Varchar>,
        query -> Nullable<Varchar>,
        raw_post -> Nullable<Varchar>,
        categories -> Nullable<Array<Text>>,
        source -> Nullable<Array<Text>>,
        providers -> Nullable<Jsonb>,
        uuid -> Uuid,
        datetime ->  Nullable<Timestamp>,
        created_at ->  Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
        api_version -> Nullable<Text>,
        organization_id -> Nullable<Int4>,
    }
}
