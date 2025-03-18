use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Queryable, Debug)]
pub struct Order {
    pub id: i32,
    pub provider_id: Option<i32>,
    pub request_id: Option<i32>,
    pub client_id: Option<i32>,
    pub organization_id: Option<i32>,
    pub obligations: Option<serde_json::Value>,
    pub state: String,
    pub declined: bool,
    pub on_hold: bool,
    pub order_cancelled: bool,
    pub request_cancelled: bool,
    pub last_computed_status: Option<String>,
    pub uuid: Uuid,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub booster: Option<f64>,
}
