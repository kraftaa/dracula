use super::prelude::*;
use rayon::prelude::*;
use std::str;

use dracula_schemas::tables::users::dsl as users_dsl;

#[derive(ParquetRecordWriter)]
struct UserRecord {
    id: i32,
    shipping_address_id: Option<i32>,
    billing_address_id: Option<i32>,
    active: String,
    company: Option<String>,
    email: Option<String>,
    full_name: Option<String>,
    sso_attributes: Option<String>,
    user_sso_email: String,
    last_request_at: Option<NaiveDateTime>,
    last_sign_in_at: Option<NaiveDateTime>,
    created_at: Option<NaiveDateTime>,
    user_agreement_signed_at: Option<NaiveDateTime>,
    expired_at: Option<NaiveDateTime>,
    uuid: Option<String>,
}

pub fn users(pg_uri: &str) -> (String, i64) {
    let conn = PgConnection::establish(pg_uri).expect("postgres conn");

    let users_load = Instant::now();
    let users = users_dsl::users.load::<User>(&conn).unwrap();
    trace!("load user took: {:?}", users_load.elapsed());

    let path = "/tmp/users.parquet";
    let records_load = Instant::now();
    let records: Vec<UserRecord> = users
        .par_iter()
        .map(|u| {
            let active = u.active.to_string();
            let sso_attributes = u.sso_attributes.as_ref().map(|l| l.clone().to_string());
            let user_sso_email = u.sso_attributes.as_ref().unwrap()["email"].to_string();

            UserRecord {
                id: u.id,
                shipping_address_id: u.shipping_address_id,
                billing_address_id: u.billing_address_id,
                active,
                company: u.company.clone(),
                email: u.email.clone(),
                full_name: u.full_name(),
                sso_attributes,
                user_sso_email,
                last_request_at: u.last_request_at,
                created_at: u.created_at,
                user_agreement_signed_at: u.user_agreement_signed_at,
                last_sign_in_at: u.last_sign_in_at,
                expired_at: u.expired_at,
                uuid: Some(u.uuid.unwrap().to_string()),
            }
        })
        .collect();

    let path_meta = <&str>::clone(&path);
    let vector_for_schema = &records;
    let schema = vector_for_schema.as_slice().schema().unwrap();
    println!("{:?} schema", &schema);

    let file = std::fs::File::create(path).unwrap();
    let mut pfile = SerializedFileWriter::new(file, schema, props()).unwrap();

    trace!(
        "load UserProviderIds ({}) took: {:?}",
        records.len(),
        records_load.elapsed()
    );

    {
        let mut row_group = pfile.next_row_group().unwrap();
        (&records[..])
            .write_to_row_group(&mut row_group)
            .expect("can't 'write_to_row_group' ...");
        pfile.close_row_group(row_group).unwrap();
    }

    pfile.close().unwrap();
    let reader = SerializedFileReader::try_from(path_meta).unwrap();
    let parquet_metadata = reader.metadata();
    let file_metadata = parquet_metadata.file_metadata();
    let rows_number = file_metadata.num_rows();
    (path.into(), rows_number)
}

pub struct UserTask {}

impl DraculaTask for UserTask {
    fn run(&self, postgres_uri: &str) -> (String, i64) {
        users(postgres_uri)
    }
}
