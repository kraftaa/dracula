use chrono::NaiveDateTime;

#[derive(Queryable, Debug)]
pub struct Rating {
    pub id: i32,
    pub value: i32,
    pub ware_id: Option<String>,
    pub spam: bool,
    pub provider_id: Option<i32>,
    pub created_at: Option<NaiveDateTime>,
    pub ware_pg_id: Option<i32>,
    pub request_id: Option<i32>,
    pub order_id: Option<i32>,
    pub comment: Option<String>,
    pub organization_id: Option<i32>,
    pub user_id: Option<i32>,
}
