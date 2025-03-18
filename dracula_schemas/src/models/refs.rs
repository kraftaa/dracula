use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Queryable, Debug)]
//#[derive(Queryable, Debug, FromSqlRow, AsExpression, Serialize, Deserialize )]
//#[sql_type = "Jsonb"]
#[allow(dead_code)]
pub struct Ref {
    // pub id: i32,
    pub id: i64,
    pub type_: Option<String>,
    pub fields: Option<serde_json::Value>,
    pub reference_of_type: Option<String>,
    pub reference_to_type: Option<String>,
    pub reference_of_id: Option<i32>,
    pub reference_to_id: Option<i32>,
    pub uuid: Uuid,
    // pub origin: Option<String>,
    // pub access: Vec<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}
//
//impl FromSql<Jsonb, Pg> for Ref {
//    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
//        let value = <serde_json::Value as FromSql<Jsonb, Pg>>::from_sql(bytes)?;
//        Ok(serde_json::from_value(value)?)
//    }
//}
//
//impl ToSql<Jsonb, Pg> for Ref {
//    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
//        let value = serde_json::to_value(self)?;
//        <serde_json::Value as ToSql<Jsonb, Pg>>::to_sql(out)
//    }
//}
