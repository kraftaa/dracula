use super::prelude::*;
use std::str;

use dracula_schemas::tables::refs::dsl as refs_dsl;

#[derive(Queryable, Debug, ParquetRecordWriter)]
struct RefsUserRecord {
    id: i64,
    type_: Option<String>,
    reference_of_type: Option<String>,
    reference_to_id: Option<i32>,
    reference_of_id: Option<i32>,
    created_at: Option<NaiveDateTime>,
    updated_at: Option<NaiveDateTime>,
    location: String,
    email: String,
    company: String,
    last_name: String,
    first_name: String,
    organization_name: String,
}

pub fn refs_users(pg_uri: &str) -> (String, i64) {
    let conn = PgConnection::establish(pg_uri).unwrap();

    let refs_load = Instant::now();

    let results = refs_dsl::refs
        .filter(refs_dsl::type_.eq("Pg::UserRef"))
        .load::<Ref>(&conn)
        .expect("Error loading refs");

    trace!("load refs took: {:?}", refs_load.elapsed());

    let path = "/tmp/refs_users.parquet";
    println!("path {:?}", &path);

    let records: Vec<RefsUserRecord> = results
        .iter()
        .map(|r| {
            let location = r
                .fields
                .as_ref()
                .unwrap()
                .get("location")
                .as_ref()
                .map(|l| (*l).clone().to_string())
                .unwrap_or_else(|| "".to_string());

            let email = r
                .fields
                .as_ref()
                .unwrap()
                .get("email")
                .as_ref()
                .map(|l| (*l).clone().to_string())
                .unwrap_or_else(|| "".to_string());
            let company = r
                .fields
                .as_ref()
                .unwrap()
                .get("company")
                .as_ref()
                .map(|l| (*l).clone().to_string())
                .unwrap_or_else(|| "".to_string());
            let last_name = r
                .fields
                .as_ref()
                .unwrap()
                .get("last_name")
                .as_ref()
                .map(|l| (*l).clone().to_string())
                .unwrap_or_else(|| "".to_string());
            let first_name = r
                .fields
                .as_ref()
                .unwrap()
                .get("first_name")
                .as_ref()
                .map(|l| (*l).clone().to_string())
                .unwrap_or_else(|| "".to_string());
            let organization_name = r
                .fields
                .as_ref()
                .unwrap()
                .get("organization_name")
                .as_ref()
                .map(|l| (*l).clone().to_string())
                .unwrap_or_else(|| "".to_string());
            let default_date =
                NaiveDateTime::parse_from_str("2020-11-26 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();

            RefsUserRecord {
                id: r.id,
                type_: r.type_.clone(),
                reference_of_type: r.reference_of_type.clone(),
                reference_to_id: r.reference_to_id,
                reference_of_id: r.reference_of_id,
                created_at: r.created_at.or(Some(default_date)),
                updated_at: r.updated_at.or(Some(default_date)),
                location: str::replace(&location, '"', ""),
                email: str::replace(&email, '"', ""),
                company: str::replace(&company, '"', ""),
                last_name: str::replace(&last_name, '"', ""),
                first_name: str::replace(&first_name, '"', ""),
                organization_name: str::replace(&organization_name, '"', ""),
            }
        })
        .collect();

    let path_meta = <&str>::clone(&path);
    let vector_for_schema = &records;
    let schema = vector_for_schema.as_slice().schema().unwrap();
    println!("{:?} schema", &schema);

    let file = std::fs::File::create(path).unwrap();
    let mut pfile = SerializedFileWriter::new(file, schema, props()).unwrap();

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

pub struct RefsUserTaskPart {}

impl DraculaTask for RefsUserTaskPart {
    fn run(&self, postgres_uri: &str) -> (String, i64) {
        refs_users(postgres_uri)
    }
}
