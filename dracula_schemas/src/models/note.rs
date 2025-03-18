use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Queryable, Debug)]
#[allow(dead_code)]
pub struct Note {
    pub id: i64,
    pub title: Option<String>,
    pub body: Option<String>,
    pub status: Option<String>,
    pub request_id: Option<i32>,
    pub ware_id: Option<String>,
    pub uuid: Uuid,
    pub user_id: Option<i32>,
    pub invoice_id: Option<i32>,
    pub purchase_order_id: Option<i32>,
    pub proposal_id: Option<i32>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}
