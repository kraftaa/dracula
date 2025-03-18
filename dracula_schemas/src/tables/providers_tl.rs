#![allow(proc_macro_derive_resolution_fallback)]

table! {
    providers (id) {
        id -> Int4,
        name -> Nullable<Varchar>,
        slug -> Nullable<Varchar>,
        uuid -> Nullable<Uuid>,
        created_at -> Nullable<Timestamp>,
        certifications -> Array<Varchar>,
        setup -> Array<Varchar>,
        score -> Double,
        tier -> Nullable<Int4>,
        contact_emails  -> Nullable<Array<Varchar>>,
        sales_email -> Nullable<Varchar>,
        phone_number -> Nullable<Varchar>,
        ads_advertiser_id -> Nullable<Int4>,
        website -> Nullable<Varchar>,
        number_of_employees -> Nullable<Varchar>,
        type_of_company -> Nullable<Varchar>,
    }
}
