#![allow(unused)]
#![allow(clippy::all)]
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Queryable, Debug)]
pub struct InventoryGroup {
    pub id: i32,
    pub index: i32,
    pub metadata: Option<serde_json::Value>,
    pub shipping_cost: BigDecimal,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub uuid: Option<Uuid>,
    pub deleted_at: Option<NaiveDateTime>,
}
