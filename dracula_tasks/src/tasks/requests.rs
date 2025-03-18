use super::prelude::*;
use std::str;
// use uuid::Uuid;

use dracula_schemas::tables::requests::dsl as requests_dsl;

#[derive(ParquetRecordWriter)]
struct RequestTaskRecord {
    id: i32,
    name: Option<String>,
    quantity: i32,
    request_type: Option<String>,
    reason_for_cancelling: Option<String>,
    filter1: Option<String>,
    filter2: Option<String>,
    filter3: Option<String>,
    closed: bool,
    cancelled: bool,
    on_hold: bool,
    status: Option<String>,
    exclude: bool,
    ordered_at: Option<NaiveDateTime>,
    user_id: Option<i32>,
    ware_id: Option<i32>,
    organization_id: Option<i32>,
    uuid: String,
    created_at: Option<NaiveDateTime>,
    updated_at: Option<NaiveDateTime>,
    description: Option<String>,
}

pub fn requests(pg_uri: &str) -> (String, i64) {
    let conn = PgConnection::establish(pg_uri).expect("postgres conn");

    let requests_load = Instant::now();
    let requests = requests_dsl::requests.load::<Request>(&conn).unwrap();
    trace!("load requests took: {:?}", requests_load.elapsed());

    let path = "/tmp/requests.parquet";
    let records_load = Instant::now();
    let records: Vec<RequestTaskRecord> = requests
        .par_iter()
        .map(|req| {
            let filter1: Option<String> = if let Some(f) = Some(req.filters.as_ref()) {
                let s = f
                    .filter(|f| !f.is_empty())
                    .map(|t| t[0].as_ref().unwrap_or(&"".to_string()).to_string())
                    .unwrap_or_else(|| "".to_string());
                Some(s)
            } else {
                None
            };

            let filter2 = if let Some(f) = Some(req.filters.as_ref()) {
                let s = f
                    .filter(|f| f.len() > 1)
                    .map(|t| t[1].as_ref().unwrap_or(&"".to_string()).to_string())
                    .unwrap_or_else(|| "".to_string());
                Some(s)
            } else {
                None
            };

            let filter3 = if let Some(f) = Some(req.filters.as_ref()) {
                let s = f
                    .filter(|f| f.len() > 2)
                    .map(|t| t[2].as_ref().unwrap_or(&"".to_string()).to_string())
                    .unwrap_or_else(|| "".to_string());
                Some(s)
            } else {
                None
            };

            let on_hold = req.on_hold;
            let cancelled = req.cancelled;
            let closed = req.closed;
            let exclude = req.exclude;

            RequestTaskRecord {
                id: req.id,
                name: req.name.clone(),
                quantity: req.quantity,
                request_type: req.request_type.clone(),
                reason_for_cancelling: req.reason_for_cancelling.clone(),
                filter1,
                filter2,
                filter3,
                cancelled,
                closed,
                on_hold,
                status: req.status.clone(),
                exclude,
                ordered_at: req.ordered_at,
                user_id: req.user_id,
                ware_id: req.ware_id,
                organization_id: req.organization_id,
                uuid: req.uuid.to_string(),
                created_at: req.created_at,
                updated_at: req.updated_at,
                description: req.description.clone(),
            }
        })
        .collect();

    let vector_for_schema = &records;
    let schema = vector_for_schema.as_slice().schema().unwrap();
    println!("{:?} schema", &schema);

    let file = std::fs::File::create(path).unwrap();
    let mut pfile = SerializedFileWriter::new(file, schema, props()).unwrap();

    trace!(
        "load QG_PG ({}) took: {:?}",
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
    let reader = SerializedFileReader::try_from(path).unwrap();
    let parquet_metadata = reader.metadata();
    let file_metadata = parquet_metadata.file_metadata();
    let rows_number = file_metadata.num_rows();
    (path.into(), rows_number)
}

pub struct RequestTask {}

impl DraculaTask for RequestTask {
    fn run(&self, postgres_uri: &str) -> (String, i64) {
        requests(postgres_uri)
    }
}
