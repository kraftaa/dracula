use chrono::NaiveDateTime;
use dracula_parquet::prelude::*;

#[derive(Queryable, Debug, ParquetRecordWriter, Default, sqlx::FromRow)]
#[allow(dead_code)]
pub struct Ware {
    pub id: i32,
    pub name: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub ware_type: Option<String>,
}
