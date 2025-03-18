#![allow(unused)]
#![allow(clippy::all)]

use chrono::NaiveDateTime;

#[derive(Queryable, Debug)]
pub struct EmbeddedDynamicForm {
    pub id: i32,
    pub name: Option<String>,
    pub version_number: Option<i32>,
    pub slug: Option<String>,
    pub key: Option<String>,
    pub schema: Option<serde_json::Value>,
    pub options: Option<serde_json::Value>,
    pub data: Option<serde_json::Value>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}
