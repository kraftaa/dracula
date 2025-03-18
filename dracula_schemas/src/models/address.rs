//use std::time::SystemTime;
use chrono::NaiveDateTime;

#[derive(Queryable, Debug)]
pub struct Address {
    pub id: i32,
    pub type_: Option<String>,
    pub organization_name: Option<String>,
    pub site_name: Option<String>,
    pub attention: Option<String>,
    pub person_name: Option<String>,
    pub care_of: Option<String>,
    pub notes: Option<String>,
    pub total: Option<String>,
    pub currency: Option<String>,
    pub address_id: Option<String>,
    pub street: Option<String>,
    pub street2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zipcode: Option<String>,
    pub country: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub name: Option<String>,
    pub site_type: Option<String>,
    pub text: Option<String>,
    pub addressable_id: Option<i32>,
    pub addressable_type: Option<String>,
    pub legal_entity_id: Option<i32>,
}

impl Address {
    pub fn to_string(&self) -> Option<String> {
        let parts: Vec<&Option<String>> = vec![
            &self.street,
            &self.street2,
            &self.city,
            &self.state,
            &self.zipcode,
            &self.country,
        ];
        let parts: Vec<String> = parts
            .into_iter()
            .flatten()
            .map(std::borrow::ToOwned::to_owned)
            .collect();
        let string = parts.join(" ");

        if !string.is_empty() {
            Some(string)
        } else {
            None
        }
    }
}
