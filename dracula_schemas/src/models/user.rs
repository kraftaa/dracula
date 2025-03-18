use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Queryable, Debug)]
pub struct User {
    pub id: i32,
    pub shipping_address_id: Option<i32>,
    pub billing_address_id: Option<i32>,
    pub active: bool,
    pub company: Option<String>,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub last_request_at: Option<NaiveDateTime>,
    pub last_sign_in_at: Option<NaiveDateTime>,
    pub created_at: Option<NaiveDateTime>,
    pub sso_attributes: Option<serde_json::Value>,
    pub user_agreement_signed_at: Option<NaiveDateTime>,
    pub expired_at: Option<NaiveDateTime>,
    pub uuid: Option<Uuid>,
}

impl User {
    // Computed property
    pub fn full_name(&self) -> Option<String> {
        let tuple = (&self.first_name, &self.last_name);

        match tuple {
            (None, None) => None,
            (Some(first), None) => Some(first.clone()),
            (None, Some(last)) => Some(last.clone()),
            (Some(first), Some(last)) => Some(format!("{} {}", first, last)),
        }
    }
}
