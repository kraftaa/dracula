use super::prelude::*;

use dracula_schemas::tables::organizations::dsl as organizations_dsl;
// use uuid::Uuid;

#[derive(ParquetRecordWriter)]
struct OrganizationRecord {
    id: i32,
    name: Option<String>,
    uuid: Option<String>,
    created_at: Option<NaiveDateTime>,
    updated_at: Option<NaiveDateTime>,
    parent_id: Option<i32>,
    domain: Option<String>,
    webstore: String,
    archived_at: Option<NaiveDateTime>,
}

pub fn organizations(pg_uri: &str) -> (String, i64) {
    let conn = PgConnection::establish(pg_uri).unwrap();

    let organizations_load = Instant::now();
    let organizations = organizations_dsl::organizations
        .load::<Organization>(&conn)
        .unwrap();
    trace!("load organization took: {:?}", organizations_load.elapsed());

    let path = "/tmp/organizations.parquet";
    // let mut pfile = dracula_parquet::parquet_writer::<OrganizationRecord>(path).unwrap();

    let records: Vec<OrganizationRecord> = organizations
        .iter()
        .map(|o| {
            let webstore = o.webstore.to_string();

            OrganizationRecord {
                id: o.id,
                name: o.name.clone(),
                uuid: Some(o.uuid.unwrap().to_string()),
                created_at: o.created_at,
                updated_at: o.updated_at,
                parent_id: o.parent_id,
                domain: o.domain.clone(),
                webstore,
                archived_at: o.archived_at,
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

pub struct OrganizationTask {}

impl DraculaTask for OrganizationTask {
    fn run(&self, postgres_uri: &str) -> (String, i64) {
        organizations(postgres_uri)
    }
}
