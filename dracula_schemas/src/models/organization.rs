use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Queryable, Debug)]
pub struct Organization {
    pub id: i32,
    pub name: Option<String>,
    pub uuid: Option<Uuid>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub parent_id: Option<i32>,
    pub domain: Option<String>,
    pub webstore: bool,
    pub archived_at: Option<NaiveDateTime>,
}
