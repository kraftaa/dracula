use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Queryable, Debug)]
pub struct Request {
    pub id: i32,
    pub name: Option<String>,
    pub description: Option<String>,
    pub quantity: i32,
    pub request_type: Option<String>,
    pub reason_for_cancelling: Option<String>,
    pub filters: Option<Vec<Option<String>>>, // -> <Array<Nullable<Varchar> >,
    pub closed: bool,
    pub cancelled: bool,
    pub on_hold: bool,
    pub status: Option<String>,
    pub exclude: bool,
    pub ordered_at: Option<NaiveDateTime>,
    pub user_id: Option<i32>,
    pub ware_id: Option<i32>,
    pub organization_id: Option<i32>,
    pub uuid: Uuid,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}
