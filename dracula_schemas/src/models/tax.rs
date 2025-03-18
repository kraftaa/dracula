use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;

#[derive(Queryable, Debug)]
pub struct Tax {
    pub id: i32,
    pub taxable_id: Option<i32>,
    pub taxable_type: Option<String>,
    pub amount: BigDecimal,
    pub category: Option<String>,
    pub currency: Option<String>,
    pub description: Option<String>,
    pub locale: Option<String>,
    pub notes: Option<String>,
    pub percent_rate: BigDecimal,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    // pub origin: Option<String>,
    // pub access: Vec<String>,
    pub type_: Option<String>,
}
