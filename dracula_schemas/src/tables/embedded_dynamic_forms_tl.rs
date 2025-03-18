table! {
    embedded_dynamic_forms (id) {
        id -> Int4,
        name -> Nullable<Varchar>,
        version_number -> Nullable<Int4>,
        slug -> Nullable<Varchar>,
        key -> Nullable<Varchar>,
        schema -> Nullable<Json>,
        options -> Nullable<Json>,
        data -> Nullable<Json>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}
