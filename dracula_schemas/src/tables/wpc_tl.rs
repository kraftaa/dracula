table! {
    wpcs (id) {
        id -> Int8,
        ware_id -> Nullable<Int4>,
        provider_id -> Nullable<Int4>,
        required -> Nullable<Bool>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
        organization_id -> Nullable<Int4>,
        auto_assigned -> Bool,
        booster -> Double,
    }
}
