use std::io::Write;
extern crate openssl;
use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::operation::list_objects_v2::{ListObjectsV2Error, ListObjectsV2Output};
use aws_sdk_s3::types::Object;
use chrono::{DateTime, Datelike, Duration, NaiveDateTime, Utc};
use diesel::{table, Queryable};
use flate2::write::GzDecoder;
use std::time::SystemTime;
use tokio_postgres::NoTls;

table! {
    clicks {
        id -> Varchar,
        data -> Nullable<Jsonb>,
        file_name -> Nullable<Varchar>,
        last_modified_date -> Nullable<Timestamp>,
        year -> Int4,
    }
}

#[derive(Insertable, Queryable)]
#[table_name = "clicks"]
pub struct Click {
    pub id: String,
    pub data: Option<serde_json::Value>,
    pub file_name: Option<String>,
    pub last_modified_date: Option<NaiveDateTime>,
    pub year: i32,
}

pub async fn load_object_from_s3(
    params: (Object, aws_sdk_s3::Client),
) -> (String, NaiveDateTime, String) {
    let (object, client) = params;

    let get_object = client
        .get_object()
        .key(object.key().unwrap())
        .bucket("domain-ads-production");

    let _start = std::time::Instant::now();
    let result = get_object.send().await.unwrap();
    let body = result.body.collect().await.unwrap();
    let last_modified_time = NaiveDateTime::from_timestamp_opt(
        result.last_modified.unwrap().secs(),
        result.last_modified.unwrap().subsec_nanos(),
    )
    .unwrap();
    (
        gz_body_to_string(body.into_bytes().to_vec()),
        last_modified_time,
        object.key().unwrap().to_string(),
    )
}
pub async fn load_clicks_from_s3(
    params: (Object, aws_sdk_s3::Client),
) -> (String, NaiveDateTime, String) {
    let (object, client) = params;
    let get_object = client
        .get_object()
        .key(object.key().unwrap())
        .bucket("domain-ads-production");

    let _start = std::time::Instant::now();
    let result = get_object.send().await.unwrap();
    let body = result.body.collect().await.unwrap();
    let last_modified_time = NaiveDateTime::from_timestamp_opt(
        result.last_modified.unwrap().secs(),
        result.last_modified.unwrap().subsec_nanos(),
    );

    (
        String::from_utf8(body.into_bytes().to_vec()).expect("String parsing error"),
        last_modified_time.unwrap(),
        object.key().unwrap().to_string(),
    )
}

pub async fn extract_contents_from_objects_output(
    objects_output: Result<ListObjectsV2Output, SdkError<ListObjectsV2Error>>,
) -> tokio_stream::Iter<std::vec::IntoIter<Object>> {
    let objects_output_ref = objects_output.as_ref().unwrap();
    println!(
        "pulling object descriptions from {} ({} entries)",
        objects_output_ref.name().unwrap(),
        objects_output_ref.contents().len(),
    );
    tokio_stream::iter(objects_output.unwrap().contents.unwrap().into_iter())
}

pub fn gz_body_to_string(body: Vec<u8>) -> String {
    let mut writer = Vec::new();
    let mut decoder = GzDecoder::new(writer);
    decoder.write_all(&body[..]).unwrap();
    writer = decoder.finish().unwrap();
    String::from_utf8(writer).expect("String parsing error")
}

pub fn last_folder_to_load() -> (i32, u32, u32) {
    let now = Utc::now();
    println!("today {}", now);
    let one_day_ago = now - Duration::days(1);
    let year_day_ago = one_day_ago.year();
    let month_day_ago = one_day_ago.month();
    let day_day_ago = one_day_ago.day();
    (year_day_ago, month_day_ago, day_day_ago)
}

pub fn last_folders_to_load(_clicks: bool) -> Vec<String> {
    let now = Utc::now();
    let mut folders = Vec::new();
    println!("today {}", now);

    for i in 0..5 {
        let days_ago = now - Duration::days(i as i64);
        let year_days_ago = days_ago.year();
        let month_days_ago = if days_ago.month() < 10 {
            // let month_days_ago = if days_ago.month() < 10 && !clicks {
            format!("0{}", days_ago.month())
        } else {
            format!("{}", days_ago.month())
        };
        let day_days_ago = if days_ago.day() < 10 {
            format!("0{}", days_ago.day())
        } else {
            format!("{}", days_ago.day())
        };
        let folder = format!("{}/{}/{}", year_days_ago, month_days_ago, day_days_ago);
        folders.push(folder);
    }
    // let last_folders = if clicks {
    //     folders[1..].to_vec()
    // } else {
    //     folders
    // };
    let last_folders = folders;

    println!("{:?} folders", &last_folders);
    last_folders
}

// pub async fn last_modified_date(pg_uri: &str) -> Vec<NaiveDateTime> {
pub async fn last_modified_date(pg_uri: &str, table_name: &str) -> Option<NaiveDateTime> {
    let (client, connection) = tokio_postgres::connect(pg_uri, NoTls).await.unwrap();
    // let mut last_modified_vector:Vec<NaiveDateTime> = vec![];
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    // let tables = vec!["impressions", "requests", "selections","decisions", "clicks"];
    // for i in tables {
    let query = format!(
        "SELECT MAX(last_modified_date) last_date FROM {}",
        table_name
    );
    let row = client.query_one(&query, &[]).await.unwrap();

    // println!("row {:?}", row);
    // println!("row len {:?}", row.len());
    let system_time = match row.try_get::<_, SystemTime>(0) {
        Ok(system_time) => {
            // println or other actions when system_time is not null
            println!("system_time {:?}", system_time);
            Some(system_time)
        }
        Err(_) => {
            println!("No records found in table {:?}", &table_name);
            None
        }
    };
    // let date_time: DateTime<Utc> = DateTime::from(system_time.unwrap());
    // let naive_date_time: Option<NaiveDateTime> = Some(date_time.naive_utc());
    // println!("Max last_modified date: {:?}", naive_date_time);
    // naive_date_time

    let naive_date_time: Option<NaiveDateTime> = system_time.map(|system_time| {
        let date_time: DateTime<Utc> = DateTime::from(system_time);
        date_time.naive_utc()
    });

    println!("Max last_modified date: {:?}", naive_date_time);
    naive_date_time

    // this option with postgress::Connection as PgConn doesn't work
    // let conn = PgConn::connect(pg_uri, &SslMode::None);
    // let conn = conn.unwrap();
    // use chrono::NaiveDateTime;
    //
    // let stmt = conn.prepare("SELECT MAX(last_modified_date) as last_date FROM impressions").unwrap();
    // let rows = stmt.query(&[]).unwrap();
    //
    // for row in &rows {
    //     println!("{:?}", row.get(0));
    // }
}

pub async fn last_modified_dates(pg_uri: &str) -> Vec<Option<NaiveDateTime>> {
    let (client, connection) = tokio_postgres::connect(pg_uri, NoTls).await.unwrap();
    let mut last_modified_vector: Vec<Option<NaiveDateTime>> = vec![];
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    let tables = vec![
        "impressions",
        "requests",
        "selections",
        "decisions",
        "clicks",
    ];
    for i in tables {
        let query = format!("SELECT MAX(last_modified_date) last_date FROM {}", i);
        let row = client.query_one(&query, &[]).await.unwrap();

        println!("row {:?}", row);
        println!("row len {:?}", row.len());
        let system_time = match row.try_get::<_, SystemTime>(0) {
            Ok(system_time) => {
                // println or other actions when system_time is not null
                println!("system_time {:?}", system_time);
                Some(system_time)
            }
            Err(_) => {
                println!("No records found in table");
                None
            }
        };
        // let system_time: SystemTime = row.get(0);
        if system_time.is_some() {
            let date_time: DateTime<Utc> = DateTime::from(system_time.unwrap());
            let naive_date_time: Option<NaiveDateTime> = Some(date_time.naive_utc());
            println!("Max last_modified date: {:?} for {:?}", naive_date_time, i);
            last_modified_vector.push(naive_date_time)
        } else {
            last_modified_vector.push(None)
        }
        // let date_time: DateTime<Utc> = DateTime::from(system_time.unwrap());
        // let naive_date_time: Option<NaiveDateTime> = Some(date_time.naive_utc());
        // println!("Max last_modified date: {:?} for {:?}", naive_date_time, i);
        // last_modified_vector.push(naive_date_time)
    }
    println!(
        "last modified dates for impressions, requests, selections, decisions, clicks {:?}",
        last_modified_vector
    );
    last_modified_vector
}
