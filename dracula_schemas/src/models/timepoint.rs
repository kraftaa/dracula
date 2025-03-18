use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Queryable, Debug)]
#[allow(dead_code)]
pub struct TimepointPG {
    pub id: i32,
    pub name: Option<String>,
    pub date: Option<NaiveDateTime>,
    pub slug: Option<String>,
    pub order_id: Option<i32>,
    pub uuid: Uuid,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}
