use super::prelude::*;
use chrono::{Duration, Utc};
use dracula_schemas::tables::events::dsl as events_dsl;
use dracula_schemas::tables::events_tl::events::columns::created_at;
use dracula_schemas::tables::events_tl::events::columns::event;
use std::collections::HashSet;
use std::path::PathBuf;
use std::str;

pub const BASE_PATH: &str = "domain-datawarehouse";
#[derive(ParquetRecordWriter, Clone, Queryable)]
struct EventEmailRecord {
    id: i64,
    event: Option<String>,
    email: Option<String>,
    username: Option<String>,
    application: Option<String>,
    duration: Option<f64>,
    remote_addr: Option<String>,
    host: Option<String>,
    action: Option<String>,
    controller: Option<String>,
    session_id: Option<String>,
    computer_id: Option<String>,
    query: Option<String>,
    raw_post: Option<String>,
    categories: String,
    source: String,
    providers: Option<String>,
    uuid: String,
    datetime: Option<NaiveDateTime>,
    created_at: Option<NaiveDateTime>,
    updated_at: Option<NaiveDateTime>,
    organization_id: Option<i32>,
    full_email: Option<String>,
}

#[allow(path_statements)]
pub fn events_emails(pg_uri: &str) -> Vec<(NaiveDate, PathBuf, u128, i64)> {
    let conn = PgConnection::establish(pg_uri).unwrap();

    let _events_load = Instant::now();

    let _current_timestamp = chrono::offset::Utc::now();

    let mut filenames: Vec<(NaiveDate, PathBuf, u128, i64)> = Vec::new();

    let now = Utc::now();
    let mut start_date = Some(now.date_naive());
    println!("original start {:?}", start_date);
    let end_date = start_date;
    start_date = Some(start_date.unwrap() - Duration::days(37));

    println!("new start -7 days {:?}", start_date);

    println!("end date {:?}", end_date);
    while start_date <= end_date {
        println!("start {:?}", start_date);
        let _current_week_number = start_date.unwrap().iso_week().week();

        let diff_days = start_date.unwrap().weekday().num_days_from_monday() as i64;
        println!("diff_days {}", &diff_days);

        let start_of_week = start_date.unwrap() - Duration::days(diff_days); //.date();
        let end_of_week = start_of_week + Duration::days(6);

        println!("start of week {:?}", start_of_week);

        let start_naive_dt = start_of_week.and_hms_opt(0, 0, 0).expect("start_naive_dt");

        let end_naive_dt = end_of_week
            .and_hms_nano_opt(23, 59, 59, 999_999_999)
            .expect("end naive dt");
        println!("{:?} naive_dt_end", end_naive_dt);
        let dracula_time = Instant::now();
        let results = events_dsl::events
            .filter(event.is_not_null())
            .filter(created_at.between(start_naive_dt, end_naive_dt))
            .load::<Event>(&conn)
            .expect("Error loading events");

        let _session_ids: Vec<String> = results
            .iter()
            .filter_map(|x| x.session_id.clone())
            .collect::<HashSet<_>>() // collect into HashSet first to remove duplicates
            .into_iter() // convert back into an Iterator
            .collect();
        let mut emails_by_session: HashMap<String, Option<String>> = HashMap::new();
        for result in &results {
            if let Some(_session_id) = &result.session_id {
                if let Some(email) = &result.email {
                    emails_by_session
                        .entry(result.session_id.as_ref().unwrap().clone())
                        .or_insert(Some(email.clone()));
                }
            }
        }

        let path = PathBuf::from(format!(
            "/tmp/events_emails_partitions-{}-{}-{}.parquet",
            start_of_week.year(),
            start_of_week.month(),
            start_of_week.day()
        ));
        let another_date = end_naive_dt + Duration::seconds(1); //DateTime<_>`

        let records: Vec<EventEmailRecord> = results
            .iter()
            .map(|m| {
                let categories = m.source.clone().unwrap_or_default().join(", ");
                let source = m.source.clone().unwrap_or_default().join(", ");
                let providers = m.providers.as_ref().map(|l| l.clone().to_string());

                let emails = m
                    .session_id
                    .as_ref()
                    .and_then(|session_id| emails_by_session.get(session_id));

                let full_email = emails.map(|x| {
                    x
                        .clone()
                        .expect("Unwrapping emails in session_ids")
                });

                EventEmailRecord {
                    id: m.id,
                    event: m.event.clone(),
                    email: m.email.clone(),
                    username: m.username.clone(),
                    application: Some("".to_string()),
                    duration: m.duration,
                    remote_addr: m.remote_addr.clone(),
                    host: m.host.clone(),
                    action: m.action.clone(),
                    controller: m.controller.clone(),
                    session_id: m.session_id.clone(),
                    computer_id: m.computer_id.clone(),
                    query: m.query.clone(),
                    raw_post: m.raw_post.clone(),
                    categories,
                    source,
                    providers,
                    uuid: m.uuid.to_string(),
                    datetime: m.datetime,
                    created_at: m.created_at,
                    updated_at: m.updated_at,
                    organization_id: m.organization_id,
                    full_email,
                }
            })
            .collect();

        let path_meta = <&str>::clone(&path.to_str().unwrap());
        let vector_for_schema = &records;
        let schema = vector_for_schema.as_slice().schema().unwrap();

        let file = std::fs::File::create(&path).unwrap();
        let mut pfile = SerializedFileWriter::new(file, schema, props()).unwrap();

        {
            let mut row_group = pfile.next_row_group().unwrap();
            (&records[..])
                .write_to_row_group(&mut row_group)
                .expect("can't 'write_to_row_group' ...");
            pfile.close_row_group(row_group).unwrap();
        }

        let dracula_time = dracula_time.elapsed().as_millis();

        pfile.close().unwrap();
        let reader = SerializedFileReader::try_from(path_meta).unwrap();
        let parquet_metadata = reader.metadata();

        let file_metadata = parquet_metadata.file_metadata();
        let rows_number = file_metadata.num_rows();
        println!("{:?} file_metadata.num_rows()", file_metadata.num_rows());

        let partitioned_file = (path.clone().to_str().unwrap().replace("/tmp/", "")).to_string();
        let _key = format!("{}/{:?}", "events_emails_organizations", &path).replace("/tmp/", "");
        let split_file: Vec<&str> = partitioned_file.split('-').collect();
        let _partitioned = format!(
            "{}/dt_year={}/dt_month={}/dt_start_week={}/{:?}",
            "events_emails_organizations_new",
            split_file[1],
            split_file[2],
            split_file[3].replace(".parquet", ""),
            &partitioned_file.as_str().replace("/tmp/", "") //.replace(".parquet", "")
        );

        filenames.push((start_of_week, path, dracula_time, rows_number));
        println!("filenames {:?}", &filenames);

        start_date = Some(another_date.date());
    }
    filenames
}

pub struct EventEmailTask {}

impl HugeTask for EventEmailTask {
    fn run(&self, postgres_uri: &str) -> Vec<(NaiveDate, PathBuf, u128, i64)> {
        events_emails(postgres_uri)
    }
}
