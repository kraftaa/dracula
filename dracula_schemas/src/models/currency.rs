use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;

#[allow(dead_code)]
#[derive(Queryable, Debug, sqlx::FromRow)]
pub struct Currency {
    pub id: i32,
    pub exchangable_id: Option<i32>,
    pub exchangable_type: Option<String>,
    pub currency: Option<String>,
    pub conversion_rate: Option<BigDecimal>,
    pub conversion_set_at: Option<NaiveDateTime>,
    pub conversion_history: Option<serde_json::Value>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}
