use super::prelude::*;
use std::path::PathBuf;
use std::str;

use dracula_schemas::tables::events::dsl as events_dsl;
use dracula_schemas::tables::events_tl::events::columns::created_at;
use dracula_schemas::tables::events_tl::events::columns::event;

#[derive(ParquetRecordWriter)]
struct EventRecord {
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
}
#[allow(path_statements)]
pub fn events(pg_uri: &str) -> Vec<(NaiveDate, PathBuf, u128, i64)> {
    let conn = PgConnection::establish(pg_uri).unwrap();

    let events_load = Instant::now();

    // let years = 2020..2021; //to rerun manually

    let current_timestamp = chrono::offset::Utc::now();
    let current_date = DateTime::date_naive(&current_timestamp);
    let current_year = DateTime::year(&current_timestamp);
    let current_month = DateTime::month(&current_timestamp);
    // let years = current_year..current_year + 1;
    let years = if current_month == 1 && current_date.day() < 8 {
        vec![current_year - 1, current_year]
    } else {
        vec![current_year]
    };
    let mut filenames: Vec<(NaiveDate, PathBuf, u128, i64)> = Vec::new();
    //    let days = vec![1, 11, 21];

    // let day = 15;  // to recalculate the whole month or starting certain day
    let mut day = current_date.day();
    println!("{:?}", &day);
    let day_module = if &day % 7 != 0 {
        &day / 7 + 1
    } else {
        &day / 7
    };

    println!("{:?} day module", &day_module);

    let weeks = if day_module == 1 {
        if current_month == 3
            && day_module == 1
            && ((current_year % 4 == 0 && current_year % 100 != 0) || current_year % 400 == 0)
        {
            vec![5, 1]
        } else {
            vec![4, 1]
        }
        // vec![5, 1]
    } else {
        vec![day_module - 1, day_module]
    };

    let months = if current_month == 1 && current_date.day() < 8 {
        vec![12, current_month]
    } else if day_module == 1 && current_month != 1 {
        vec![current_month - 1, current_month]
    } else {
        vec![current_month]
    };

    let long_months = vec![1, 3, 5, 7, 8, 10, 12];
    for year in years {
        for month in &months {
            if month == &0 {
                continue;
            }
            // for month in 1..7 {   // to recalculate months
            let mut last_day = 31;
            if *month == 2 {
                if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
                    last_day = 29;
                } else {
                    last_day = 28;
                }
            } else if !long_months.contains(month) {
                last_day = 30;
            }

            let last_day_month = NaiveDate::from_ymd_opt(year, *month, last_day);
            println!("{:?} last day month", &last_day_month);
            for week in weeks.clone() {
                if year == current_year
                    && month == &2_u32
                    && last_day_month.expect("REASON").day() == 28_u32
                    && week == 5_u32
                {
                    println!("{} {} {}", current_date, month, week);
                    continue;
                } else {
                    let first_day = if week == 1 {
                        NaiveDate::from_ymd_opt(year, *month, 1).expect("first day")
                    } else if week * 7 > last_day_month.expect("no last day").day() {
                        day = (week - 1) * 7 + 1;

                        NaiveDate::from_ymd_opt(year, *month, day).unwrap()
                    } else {
                        NaiveDate::from_ymd_opt(year, *month, (week - 1) * 7 + 1).unwrap()
                    };
                    println!("{:?} first day", &first_day);
                    //                first_day = NaiveDate::from_ymd(year, month, *week);
                    let last_day_calculation = if week * 7 < last_day {
                        NaiveDate::from_ymd_opt(year, *month, (week) * 7)
                            .expect("last_day_calculation")
                    } else {
                        NaiveDate::from_ymd_opt(year, *month, last_day)
                            .expect("last_day_calculation")
                    };

                    println!("{:?} first day", &first_day);
                    println!("{:?} last day calc", &last_day_calculation);
                    let dracula_time = Instant::now();
                    let results = events_dsl::events
                        .filter(event.is_not_null())
                        .filter(created_at.between(
                            first_day.and_hms_opt(0, 0, 0),
                            last_day_calculation.and_hms_nano_opt(23, 59, 59, 999_999_999),
                        )) // this doesn't complain either
                        .load::<Event>(&conn)
                        .expect("Error loading events");
                    println!("after results");
                    trace!("load events took: {:?}", events_load.elapsed());

                    let path = PathBuf::from(format!(
                        "/tmp/events_partitions-{}-{}-{}.parquet",
                        year, month, week
                    ));
                    println!("path {:?}", &path);

                    println!("before unwrap1");
                    // let mut pfile =
                    //     dracula_parquet::parquet_writer_path::<EventRecord>(path.as_path())
                    //         .unwrap();
                    // println!("after pfile");

                    let records: Vec<EventRecord> = results
                        .iter()
                        .map(|m| {
                            let categories = m.source.clone().unwrap_or_default().join(", ");
                            let source = m.source.clone().unwrap_or_default().join(", ");
                            let providers = m.providers.as_ref().map(|l| l.clone().to_string());

                            EventRecord {
                                id: m.id,
                                event: m.event.clone(),
                                email: m.email.clone(),
                                username: m.username.clone(),
                                application: m.application.clone(),
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
                            }
                            //            }
                        })
                        .collect();

                    let path_meta = <&str>::clone(&path.to_str().unwrap());
                    let vector_for_schema = &records;
                    let schema = vector_for_schema.as_slice().schema().unwrap();
                    println!("{:?} schema", &schema);
                    // let props = Arc::new(WriterProperties::builder().build());

                    let file = std::fs::File::create(&path).unwrap();
                    let mut pfile = SerializedFileWriter::new(file, schema, props()).unwrap();

                    {
                        let mut row_group = pfile.next_row_group().unwrap();
                        (&records[..])
                            .write_to_row_group(&mut row_group)
                            .expect("can't 'write_to_row_group' ...");
                        pfile.close_row_group(row_group).unwrap();
                    }

                    // let rows_number = *pfile.total_num_rows() as i64;
                    let dracula_time = dracula_time.elapsed().as_millis();
                    // filenames.push((first_day, path, dracula_time, rows_number));
                    // println!("filenames {:?}", &filenames);
                    pfile.close().unwrap();
                    let reader = SerializedFileReader::try_from(path_meta).unwrap();
                    let parquet_metadata = reader.metadata();
                    println!("{:?} num row group", parquet_metadata.num_row_groups());

                    let file_metadata = parquet_metadata.file_metadata();
                    let rows_number = file_metadata.num_rows();
                    println!("{:?} file_metadata.num_rows()", file_metadata.num_rows());
                    filenames.push((first_day, path, dracula_time, rows_number));
                    println!("filenames {:?}", &filenames);
                    if last_day_calculation >= last_day_month.expect("last_day_month") {
                        continue;
                    } else {
                        // TODO: Maria, take a look at this!
                        let _first_day = NaiveDate::from_ymd_opt(year, *month, (week) * 7 + 1)
                            .expect("first day");
                    }
                } // day
            } // week
        } // month
    }
    filenames
}

pub struct EventTask {}

impl HugeTask for EventTask {
    fn run(&self, postgres_uri: &str) -> Vec<(NaiveDate, PathBuf, u128, i64)> {
        events(postgres_uri)
    }
}
