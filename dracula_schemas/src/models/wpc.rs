use chrono::NaiveDateTime;

#[derive(Queryable, Debug)]
pub struct WPC {
    pub id: i64,
    pub ware_id: Option<i32>,
    pub provider_id: Option<i32>,
    pub required: Option<bool>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub organization_id: Option<i32>,
    pub auto_assigned: bool,
    pub booster: f64,
}
