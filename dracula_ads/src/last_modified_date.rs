use chrono::{DateTime, Datelike, Duration, LocalResult, NaiveDateTime, TimeZone, Utc};
use std::time::SystemTime;
use tokio_postgres::NoTls;

pub async fn last_folder() {
    let now = Utc::now();
    let year = now.year();
    let month = now.month();
    let day = now.day();
    // let dt: DateTime<Utc> = Utc.ymd(2020, 3, 1).and_hms(0, 0, 0) - Duration::days(1);
    let date_time = Utc.with_ymd_and_hms(2020, 3, 1, 0, 0, 0);
    let date = match date_time {
        LocalResult::Single(date) => date,
        _ => panic!("wrong data format"),
    };
    let dt: DateTime<Utc> = date - Duration::days(1);
    // chrono::LocalResult<chrono::DateTime<chrono::Utc>>

    let one_day_ago = dt;
    // let one_day_ago = now - Duration::days(1);
    let year_day_ago = one_day_ago.year();
    let month_day_ago = one_day_ago.month();
    let day_day_ago = one_day_ago.day();
    println!(
        "what is that actually {} {} {} {} {} {} {}",
        now, year, month, day, year_day_ago, month_day_ago, day_day_ago
    );
}

// pub async fn last_modified_date(pg_uri: &str, table_name: &str) -> Vec<NaiveDateTime> {
pub async fn last_modified_date(pg_uri: &str, table_name: &str) -> NaiveDateTime {
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

    println!("row {:?}", row);
    let system_time: SystemTime = row.get(0);
    let date_time: DateTime<Utc> = DateTime::from(system_time);
    let naive_date_time: NaiveDateTime = date_time.naive_utc();
    println!(
        "Max last_modified date: {:?} for {:?}",
        naive_date_time, table_name
    );
    // last_modified_vector.push(naive_date_time)
    // }
    // last_modified_vector
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
