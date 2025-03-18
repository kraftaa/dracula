use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;

#[derive(Queryable, Debug)]
pub struct Milestone {
    pub id: i32,
    pub itemizable_id: Option<i32>,
    pub itemizable_type: Option<String>,
    pub unit_price: BigDecimal,
    pub total_price: BigDecimal,
    pub _type: String,
    pub name: Option<String>,
    pub quantity: BigDecimal,
    pub state: String,
    pub tax_rate: BigDecimal,
    pub currency: Option<String>,
    pub unit_of_measure: Option<String>,
    pub classifications: serde_json::Value,
    pub line_number: Option<i32>,
    pub comments: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub status: Option<String>,
    pub provider_id: Option<i32>,
    pub cancelled_at: Option<NaiveDateTime>,
    pub shipped_at: Option<NaiveDateTime>,
    pub estimated_date: Option<NaiveDateTime>,
}

impl Milestone {
    pub fn total_tax_amount(&self) -> BigDecimal {
        self.total_price.clone() * (self.tax_rate.clone() / BigDecimal::from(100))
    }
}
