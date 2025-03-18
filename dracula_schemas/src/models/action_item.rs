use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Queryable, Debug)]
pub struct ActionItem {
    pub id: i64,
    pub action_performed: Option<String>,
    pub completed_at: Option<NaiveDateTime>,
    pub proposal_id: Option<i32>,
    pub purchase_order_id: Option<i32>,
    pub provider_id: Option<String>,
    pub uuid: Option<Uuid>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub order_id: Option<i32>,
    pub request_id: Option<i32>,
    pub note_id: Option<i32>,
    pub invoice_id: Option<i32>,
    pub request_name: Option<String>,
    pub organization_id: Option<i32>,
    pub type_: Option<String>,
    pub ads_ad_id: Option<i32>,
}
