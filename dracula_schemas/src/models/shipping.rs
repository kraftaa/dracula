use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;

#[derive(Queryable, Debug)]
pub struct Shipping {
    pub id: i32,
    pub shipable_id: Option<i32>,
    pub shipable_type: Option<String>,
    pub cost: BigDecimal,
    pub free_shipping: bool,
    pub currency: Option<String>,
    pub notes: Option<String>,
    // pub origin: Option<String>,
    // pub access: Option<Vec<String>>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}
