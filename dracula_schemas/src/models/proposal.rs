use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Queryable, Debug)]
pub struct Proposal {
    pub id: i32,
    pub active: bool,
    pub description: String,
    pub obsoleted: bool,
    pub proposal_type: Option<String>,
    pub state: Option<String>,
    pub exclude: bool,
    pub justifications: Option<Vec<String>>,
    pub commission_rate: Option<BigDecimal>,
    pub user_full_name: Option<String>,
    pub user_id: Option<i32>,
    pub uuid: Option<Uuid>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub subtotal_price: BigDecimal,
    pub total_price: BigDecimal,
    pub provider_id: Option<i32>,
    pub tax_category: Option<String>,
    pub order_id: Option<i32>,
    pub type_: Option<String>,
    pub ware_id: Option<i32>,
    pub status: Option<String>,
}
