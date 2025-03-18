#![allow(proc_macro_derive_resolution_fallback)]

table! {
    addresses (id) {
        id -> Int4,
        #[sql_name = "type"]
        type_ -> Nullable<Varchar>,
        organization_name -> Nullable<Varchar>,
        site_name -> Nullable<Varchar>,
        attention -> Nullable<Varchar>,
        person_name -> Nullable<Varchar>,
        care_of -> Nullable<Varchar>,
        notes -> Nullable<Text>,
        total -> Nullable<Varchar>,
        currency -> Nullable<Varchar>,
        address_id -> Nullable<Varchar>,
        street -> Nullable<Varchar>,
        street2 -> Nullable<Varchar>,
        city -> Nullable<Varchar>,
        state -> Nullable<Varchar>,
        zipcode -> Nullable<Varchar>,
        country -> Nullable<Varchar>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
        name -> Nullable<Varchar>,
        site_type -> Nullable<Varchar>,
        text -> Nullable<Text>,
        addressable_id -> Nullable<Int4>,
        addressable_type -> Nullable<Varchar>,
        legal_entity_id -> Nullable<Int4>,
    }
}
