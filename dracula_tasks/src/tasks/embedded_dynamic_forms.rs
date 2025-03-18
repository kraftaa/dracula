use super::prelude::*;

use dracula_schemas::tables::embedded_dynamic_forms::dsl as embedded_dynamic_forms_dsl;
use dracula_schemas::tables::embedded_dynamic_forms_tl::embedded_dynamic_forms::columns::created_at;
use std::path::PathBuf;

#[derive(ParquetRecordWriter)]
struct EmbeddedDynamicFormRecord {
    id: i32,
    name: Option<String>,
    version_number: Option<i32>,
    slug: Option<String>,
    key: Option<String>,
    schema: Option<String>,
    options: Option<String>,
    data: Option<String>,
    created_at: Option<NaiveDateTime>,
    updated_at: Option<NaiveDateTime>,
}
pub fn embedded_dynamic_forms(pg_uri: &str) -> Vec<(NaiveDate, PathBuf, u128, i64)> {
    let conn = PgConnection::establish(pg_uri).unwrap();

    let embedded_dynamic_forms_load = Instant::now();

    let current_timestamp = chrono::offset::Utc::now();
    let current_date = DateTime::date_naive(&current_timestamp);
    let current_month = DateTime::month(&current_timestamp);
    let current_year = DateTime::year(&current_timestamp);

    let years = if current_month == 1 && current_date.day() < 8 {
        vec![current_year - 1, current_year]
    } else {
        vec![current_year]
    };
    let mut filenames: Vec<(NaiveDate, PathBuf, u128, i64)> = Vec::new();

    let mut day = current_date.day();
    // let day = 1; // to recalculate
    println!("{:?}", &day);
    let day_module = if &day % 7 != 0 {
        // uncomment for regular count
        &day / 7 + 1
    } else {
        &day / 7
    };

    // let  day_module = 1;  // for full recalculation and upload
    println!("{:?} day module", &day_module);

    // let weeks = day_module..day_module + 1; // for current week // uncomment for regular count
    let weeks = if day_module == 1 {
        // day_module..day_module + 1
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
            fn is_leap_year(y: i32) -> bool {
                y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)
            }
            // for month in current_month..current_month + 1 { // uncomment for regular count
            // for month in 1..13 {
            let mut last_day = 31;
            if *month == 2 {
                if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
                    last_day = 29;
                    // weeks = 1..5;
                } else {
                    last_day = 28;
                }
            } else if !long_months.contains(month) {
                last_day = 30;
            }

            let last_day_month = NaiveDate::from_ymd_opt(year, *month, last_day).unwrap();
            println!("{:?} last day month", &last_day_month);

            for week in weeks.clone() {
                if !is_leap_year(year) && month == &2 && week == 5 {
                    // if year == current_year && month == &(2 as u32) && &last_day_month.day() == &(28 as u32) && &week == &(5 as u32) {
                    println!("{} {} {}", current_date, month, week);
                    continue;
                } else {
                    let first_day = if week == 1 {
                        NaiveDate::from_ymd_opt(year, *month, 1)
                    } else if week * 7 > last_day_month.day() {
                        day = (week - 1) * 7 + 1;

                        NaiveDate::from_ymd_opt(year, *month, day)
                    } else {
                        NaiveDate::from_ymd_opt(year, *month, (week - 1) * 7 + 1)
                    };

                    let last_day_calculation = if week * 7 < last_day {
                        NaiveDate::from_ymd_opt(year, *month, (week) * 7)
                    } else {
                        NaiveDate::from_ymd_opt(year, *month, last_day)
                    };

                    println!("{:?} first day", &first_day);
                    println!("{:?} last day calc", &last_day_calculation);

                    let dracula_time = Instant::now();
                    let embedded_dynamic_forms = embedded_dynamic_forms_dsl::embedded_dynamic_forms
                        .filter(
                            created_at.between(
                                first_day.expect("first day").and_hms_opt(0, 0, 0),
                                last_day_calculation
                                    .expect("last_day_calculation")
                                    .and_hms_nano_opt(23, 59, 59, 999_999_999)
                                    .expect("last day calculation"),
                            ),
                        ) // this doesn't complain either
                        .load::<EmbeddedDynamicForm>(&conn)
                        .unwrap();
                    trace!(
                        "embedded_dynamic_forms: {:?}",
                        embedded_dynamic_forms_load.elapsed()
                    );

                    let path = PathBuf::from(format!(
                        "/tmp/embedded_dynamic_forms_partitions-{}-{}-{}.parquet",
                        year, month, week
                    ));
                    let path_meta = <&str>::clone(&path.to_str().unwrap());

                    let records: Vec<EmbeddedDynamicFormRecord> = embedded_dynamic_forms
                        .iter()
                        .map(|d| {
                            let schema = d.schema.as_ref().map(|l| l.clone().to_string());
                            let options = d.options.as_ref().map(|l| l.clone().to_string());
                            let data = d.data.as_ref().map(|l| l.clone().to_string());

                            EmbeddedDynamicFormRecord {
                                id: d.id,
                                name: d.name.clone(),
                                version_number: d.version_number,
                                slug: d.slug.clone(),
                                key: d.key.clone(),
                                schema,
                                options,
                                data,
                                created_at: d.created_at,
                                updated_at: d.updated_at,
                            }
                        })
                        .collect();

                    let vector_for_schema = &records;
                    let schema = vector_for_schema.as_slice().schema().unwrap();
                    // println!("{:?} schema", &schema);
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
                    let dracula_time = dracula_time.elapsed().as_millis();
                    // let rows_number = *pfile.total_num_rows() as i64;
                    // filenames.push((first_day, path, dracula_time, rows_number));
                    // println!("filenames {:?}", &filenames);
                    pfile.close().unwrap();
                    let reader = SerializedFileReader::try_from(path_meta).unwrap();
                    let parquet_metadata = reader.metadata();
                    println!("{:?} num row group", parquet_metadata.num_row_groups());

                    let file_metadata = parquet_metadata.file_metadata();
                    let rows_number = file_metadata.num_rows();
                    println!("{:?} file_metadata.num_rows()", file_metadata.num_rows());
                    filenames.push((
                        first_day.expect("first day"),
                        path,
                        dracula_time,
                        rows_number,
                    ));
                    println!("filenames {:?}", &filenames);

                    if last_day_calculation >= Some(last_day_month) {
                        continue;
                    } else {
                        let _first_day = NaiveDate::from_ymd_opt(year, *month, (week) * 7 + 1)
                            .expect("first day");
                    }
                } // day
            }
        } // month
    }
    filenames

    // pfile.close().unwrap();
    // path.into()
}

pub struct EmbeddedDynamicFormTask {}

impl HugeTask for EmbeddedDynamicFormTask {
    fn run(&self, postgres_uri: &str) -> Vec<(NaiveDate, PathBuf, u128, i64)> {
        embedded_dynamic_forms(postgres_uri)
    }
}
