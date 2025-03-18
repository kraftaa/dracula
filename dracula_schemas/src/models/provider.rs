use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Queryable, Debug)]
pub struct Provider {
    pub id: i32,
    pub name: Option<String>,
    pub slug: Option<String>,
    pub uuid: Option<Uuid>,
    pub created_at: Option<NaiveDateTime>,
    pub certifications: Vec<String>,
    pub setup: Vec<String>,
    pub score: f64,
    pub tier: Option<i32>,
    pub contact_emails: Option<Vec<String>>,
    pub sales_email: Option<String>,
    pub phone_number: Option<String>,
    pub ads_advertiser_id: Option<i32>,
    pub website: Option<String>,
    pub number_of_employees: Option<String>,
    pub type_of_company: Option<String>,
}
