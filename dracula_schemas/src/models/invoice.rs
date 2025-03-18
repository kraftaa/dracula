use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use uuid::Uuid;
#[derive(Queryable, Debug)]
pub struct Invoice {
    pub id: i32,
    pub invoice_number: Option<String>,
    pub cancelled: bool,
    pub issued_at: Option<NaiveDateTime>,
    pub document_type: String,
    pub commission_rate: Option<BigDecimal>,
    pub purchase_order_id: Option<i32>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub uuid: Option<Uuid>,
    pub subtotal_price: BigDecimal,
    pub total_price: BigDecimal,
    pub discount_amount: BigDecimal,
    pub provider_id: Option<i32>,
    pub order_id: Option<i32>,
    pub approved_for_payment: Option<bool>,
    pub discount: Option<String>,
}
