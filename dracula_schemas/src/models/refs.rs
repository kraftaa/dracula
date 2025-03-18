use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Queryable, Debug)]
pub struct Ref {
    pub id: i64,
    pub type_: Option<String>,
    pub fields: Option<serde_json::Value>,
    pub reference_of_type: Option<String>,
    pub reference_to_type: Option<String>,
    pub reference_of_id: Option<i32>,
    pub reference_to_id: Option<i32>,
    pub uuid: Uuid,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}
