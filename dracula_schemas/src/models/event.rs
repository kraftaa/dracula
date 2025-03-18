use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Queryable, Debug)]
pub struct Event {
    pub id: i64,
    pub event: Option<String>,
    pub email: Option<String>,
    pub username: Option<String>,
    pub application: Option<String>,
    pub duration: Option<f64>,
    pub remote_addr: Option<String>,
    pub host: Option<String>,
    pub action: Option<String>,
    pub controller: Option<String>,
    pub session_id: Option<String>,
    pub computer_id: Option<String>,
    pub query: Option<String>,
    pub raw_post: Option<String>,
    pub categories: Option<Vec<String>>,
    pub source: Option<Vec<String>>,
    pub providers: Option<serde_json::Value>,
    pub uuid: Uuid,
    pub datetime: Option<NaiveDateTime>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub api_version: Option<String>,
    pub organization_id: Option<i32>,
}
