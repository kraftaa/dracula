use async_std::io::Error;
/// https://url.spec.whatwg.org/#fragment-percent-encode-set
use chrono::{DateTime, Datelike};
use docopt::Docopt;
use dracula_aws::aws::*;
use dracula_cli::*;
use dracula_parquet::prelude::*;
use dracula_tasks::tasks::DraculaStreamingTask;
use dracula_tasks::tasks::DraculaTask;
use dracula_tasks::tasks::HugeTask;
use log::*;
use pretty_bytes::converter::convert;

use dracula_ads::clicks::insert_ads_clicks;
use dracula_booster::booster::insert_postgres;

use std::collections::HashMap;
use std::fs::File;
use std::panic;
use std::path::Path;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;
use std::time::Instant;
use std::{fs, str};
use url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};

async fn booster_command() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());
    insert_postgres(args.arg_POSTGRES_URI.as_str()).await
}

async fn ads_clicks() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());
    match insert_ads_clicks(args.arg_POSTGRES_URI.as_str()).await {
        Ok(_) => {}
        Err(e) => eprintln!("Error occurred: {}", e),
    }
}

fn all(_task_name: String) -> (f64, i32) {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());
    let tasks = tasks_list();
    let upload_size = AtomicUsize::new(0);
    let count = AtomicUsize::new(0);

    for (name, task) in tasks {
        println!("{:?} task ", name);
        let result = panic::catch_unwind(|| {
            let _dracula_result = task.run(
                utf8_percent_encode(args.arg_POSTGRES_URI.as_str(), DEFAULT_ENCODE_SET)
                    .to_string()
                    .as_str(),
            );
            let path = format!("{}/{}/{}.parquet", MAIN_FOLDER, name, name);
            // let dracula_file = format!("/tmp/parquet/{}.parquet", name);
            let dracula_file = format!("/tmp/{}.parquet", name);

            println!("{:?} file", &dracula_file);
            let dracula_file_for_metadata = &dracula_file;
            // let rows_len = &dracula_result.1;
            count.store(count.load(SeqCst) + 1, SeqCst);
            let metadata = fs::metadata(dracula_file_for_metadata).unwrap();
            upload_size.store(upload_size.load(SeqCst) + metadata.len() as usize, SeqCst);

            trace!("file {} size ", metadata.len());

            trace!("dracula file {}", dracula_file);
            tokio::task::spawn(async move {
                upload(dracula_file.into(), BASE_PATH, path.as_str())
                    .await
                    .expect("upload file to s3 from all - must work");
            });
        });

        if result.is_err() {
            let message = format!("Dracula GOT Panic! in {} file", name);
            info!("{}", message);
            sentry::capture_message(message.as_str(), sentry::Level::Warning);
            info!("had an error while working on {}", name);
        }
    }
    println!("finally");
    (upload_size.load(SeqCst) as f64, count.load(SeqCst) as i32)
}

async fn events() -> (f64, i64) {
    let upload_size = AtomicUsize::new(0);
    let count = AtomicUsize::new(0);
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    let event_tasks = event_tasks_list();
    let event_tasks_by_name: HashMap<&&str, &Box<dyn HugeTask>> = event_tasks
        .iter()
        .map(|(name, task)| (name, task))
        .collect();

    let current_timestamp = chrono::offset::Utc::now();
    let _current_date = DateTime::date_naive(&current_timestamp);
    let _current_year = DateTime::year(&current_timestamp);
    let _current_month = DateTime::month(&current_timestamp);

    let a_task = event_tasks_by_name.get(&&args.flag_table[..]);

    let file_name = &args.flag_table;
    if let Some(task) = a_task {
        let _dracula_time = Instant::now();
        let dracula_files = task.run(
            utf8_percent_encode(args.arg_POSTGRES_URI.as_str(), DEFAULT_ENCODE_SET)
                .to_string()
                .as_str(),
        );

        let _dracula_file_for_message = &dracula_files;

        #[allow(unused)]
        let mut rows_len = 0_i64;
        for (_date, file_path, _time_millis, _rows_number) in dracula_files {
            let copy_path = &file_path.clone();
            let key = format!("{}/{}/{:?}", MAIN_FOLDER, file_name, file_path).replace("/tmp/", "");
            // rows_len = _rows_number;
            let dracula_file_for_metadata = (file_path).to_str().unwrap();
            count.store(count.load(SeqCst) + 1, SeqCst);

            let metadata = fs::metadata(dracula_file_for_metadata).unwrap();
            upload_size.store(upload_size.load(SeqCst) + metadata.len() as usize, SeqCst);

            upload(file_path, BASE_PATH, key.as_str())
                .await
                .expect("upload file to s3");
            info!("file {} has been uploaded", file_name);
            let copy_file = copy_path
                .to_str()
                .unwrap()
                .replace("/tmp/", "")
                .replace(".parquet", "");
            let split_file: Vec<&str> = copy_file.split('-').collect();
            let copy_dir = "events_partitioned";

            let partitioned = format!(
                "{}/dt={}-{}-{}/{:?}",
                copy_dir,
                split_file[1],
                split_file[2],
                split_file[3],
                copy_path.to_str().unwrap().replace("/tmp/", "")
            );
            let from_file = key.to_string();

            copy(BASE_PATH, from_file.as_str(), partitioned.as_str())
                .await
                .unwrap();
        }

        let s3_path = format!("{}/{}/{}", BASE_PATH, MAIN_FOLDER, file_name);
        update_crawler(DATABASE.to_string(), CRAWLER_NAME_ONE.to_string(), s3_path)
            .await
            .expect("update crawler");
        start_crawler(CRAWLER_NAME_ONE.to_string(), true)
            .await
            .expect("start crawler");
    } else {
        error!("sorry, I don't know the task {}", args.flag_table)
    }
    (upload_size.load(SeqCst) as f64, count.load(SeqCst) as i64)
}

async fn events_emails() -> (f64, i64) {
    let upload_size = AtomicUsize::new(0);
    let count = AtomicUsize::new(0);
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    let event_tasks = events_emails_tasks_list();
    let event_tasks_by_name: HashMap<&&str, &Box<dyn HugeTask>> = event_tasks
        .iter()
        .map(|(name, task)| (name, task))
        .collect();

    let current_timestamp = chrono::offset::Utc::now();
    let _current_date = DateTime::date_naive(&current_timestamp);
    let _current_year = DateTime::year(&current_timestamp);
    let _current_month = DateTime::month(&current_timestamp);

    let a_task = event_tasks_by_name.get(&&args.flag_table[..]);

    let file_name = &args.flag_table;
    if let Some(task) = a_task {
        let _dracula_time = Instant::now();
        let dracula_files = task.run(
            utf8_percent_encode(args.arg_POSTGRES_URI.as_str(), DEFAULT_ENCODE_SET)
                .to_string()
                .as_str(),
        );

        let _dracula_file_for_message = &dracula_files;

        #[allow(unused)]
        let mut rows_len = 0_i64;
        for (_date, file_path, _time_millis, _rows_number) in dracula_files {
            let key =
                format!("{}/{:?}", "events_emails_organizations", file_path).replace("/tmp/", "");
            let dracula_file_for_metadata = (file_path).to_str().unwrap();
            count.store(count.load(SeqCst) + 1, SeqCst);

            let metadata = fs::metadata(dracula_file_for_metadata).unwrap();
            upload_size.store(upload_size.load(SeqCst) + metadata.len() as usize, SeqCst);
            let partitioned_file =
                (file_path.clone().to_str().unwrap().replace("/tmp/", "")).to_string();
            println!("{:?} file_path", &file_path);
            println!("{:?} key", &key);
            let split_file: Vec<&str> = partitioned_file.split('-').collect();
            let partitioned = format!(
                "{}/dt_year={}/dt_month={}/dt_start_week={}/{:?}",
                "events_emails_organizations",
                split_file[1],
                split_file[2],
                split_file[3].replace(".parquet", ""),
                &partitioned_file.as_str().replace("/tmp/", "") //.replace(".parquet", "")
            );
            println!("partitioned {:?}", &partitioned);

            upload(file_path, BASE_PATH, partitioned.as_str())
                .await
                .expect("upload file to s3");
            println!("file {} has been uploaded", file_name);
        }
    } else {
        error!("sorry, I don't know the task {}", args.flag_table)
    }
    (upload_size.load(SeqCst) as f64, count.load(SeqCst) as i64)
}

async fn embedded_dynamic_forms() -> (f64, i64) {
    let upload_size = AtomicUsize::new(0);
    let count = AtomicUsize::new(0);
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    let embedded_dynamic_form_tasks = embedded_dynamic_form_tasks_list();
    let embedded_dynamic_form_tasks_by_name: HashMap<&&str, &Box<dyn HugeTask>> =
        embedded_dynamic_form_tasks
            .iter()
            .map(|(name, task)| (name, task))
            .collect();

    let _current_timestamp = chrono::offset::Utc::now();
    let _current_date = DateTime::date_naive(&_current_timestamp);
    let _current_year = DateTime::year(&_current_timestamp);
    let _current_month = DateTime::month(&_current_timestamp);

    let a_task = embedded_dynamic_form_tasks_by_name.get(&&args.flag_table[..]);

    let file_name = &args.flag_table;

    if let Some(task) = a_task {
        let _dracula_time = Instant::now();
        let dracula_files = task.run(
            utf8_percent_encode(args.arg_POSTGRES_URI.as_str(), DEFAULT_ENCODE_SET)
                .to_string()
                .as_str(),
        );

        let _dracula_file_for_message = &dracula_files;
        for (_date, file_path, _time_millis, _rows_number) in dracula_files {
            let key = format!("{}/{}/{:?}", MAIN_FOLDER, file_name, file_path).replace("/tmp/", "");

            let dracula_file_for_metadata = (file_path).to_str().unwrap();

            count.store(count.load(SeqCst) + 1, SeqCst);

            let metadata = fs::metadata(dracula_file_for_metadata).unwrap();
            upload_size.store(upload_size.load(SeqCst) + metadata.len() as usize, SeqCst);
            // let partitioned_file = (file_path.clone().to_str().unwrap()).to_string();
            upload(file_path, BASE_PATH, key.as_str())
                .await
                .expect("upload file to s3");
            info!("file {} has been uploaded", file_name);
        }

        let s3_path = format!("{}/{}/{}", BASE_PATH, MAIN_FOLDER, file_name);
        update_crawler(DATABASE.to_string(), CRAWLER_NAME_ONE.to_string(), s3_path)
            .await
            .expect("update crawler");
        start_crawler(CRAWLER_NAME_ONE.to_string(), true)
            .await
            .expect("start crawler");
        let _file_count = count.load(SeqCst) as i32;
        let _file_size = upload_size.load(SeqCst) as f64;
    } else {
        error!("sorry, I don't know the task {}", args.flag_table)
    }
    (upload_size.load(SeqCst) as f64, count.load(SeqCst) as i64)
}

async fn single_task() {
    // let dracula_time = Instant::now();
    let _upload_size = AtomicUsize::new(0);
    let _count = AtomicUsize::new(0);
    println!("{:?} usage", USAGE);
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());
    println!("{:?} args", args);
    let tasks = tasks_list();
    let tasks_by_name: HashMap<&&str, &Box<dyn DraculaTask>> =
        tasks.iter().map(|(name, task)| (name, task)).collect();
    let a_task = tasks_by_name.get(&&args.flag_table[..]);
    let file_name = &args.flag_table;
    match a_task {
        Some(task) => {
            let dracula_file = task.run(
                utf8_percent_encode(args.arg_POSTGRES_URI.as_str(), DEFAULT_ENCODE_SET)
                    .to_string()
                    .as_str(),
            );
            let path = format!("{}/{}/{}.parquet", MAIN_FOLDER, file_name, file_name); //=> purchase_orders/purchase_orders.parquet
            println!("{:#?} file", &dracula_file);
            let dracula_file_for_metadata = &dracula_file.0;
            let rows_len = &dracula_file.1;
            let _dracula_file_for_message = dracula_file.clone();
            let metadata = fs::metadata(dracula_file_for_metadata).unwrap();

            println!("{:#?} meta", metadata);
            println!("{:#?} rows_len", rows_len);
            println!("{:#?} meta bytes", metadata.len());

            let file1 = File::open(Path::new(&dracula_file.0)).unwrap();
            let reader = SerializedFileReader::new(file1).unwrap();
            let iter = reader.get_row_iter(None).unwrap();
            println!("{:?} lines", iter.count());
            info!("file {} size ", convert(metadata.len() as f64));
            println!("{:#?} rows_len", &path);
            upload(dracula_file.0.into(), BASE_PATH, path.as_str())
                .await
                .expect("upload file to s3");
            info!("file {} has been uploaded", file_name);
            let s3_path = format!("{}/{}/{}", BASE_PATH, MAIN_FOLDER, file_name);
            futures::executor::block_on(update_crawler(
                DATABASE.to_string(),
                CRAWLER_NAME_ONE.to_string(),
                s3_path,
            ))
            .expect("update crawler");
            start_crawler(CRAWLER_NAME_ONE.to_string(), true)
                .await
                .expect("start crawler");
        }
        _ => {
            error!("sorry, I don't know the task {}", args.flag_table)
        }
    }
}

async fn inventories_task() {
    let _upload_size = AtomicUsize::new(0);
    let _count = AtomicUsize::new(0);
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    let tasks = inventories_list();
    let tasks_by_name: HashMap<&&str, &Box<dyn DraculaStreamingTask>> =
        tasks.iter().map(|(name, task)| (name, task)).collect();

    let a_task = tasks_by_name.get(&&args.flag_table[..]);
    let file_name = &args.flag_table;
    let pass_list = file_name;
    println!("{:?} pass ", pass_list);
    // let pgi = utf8_percent_encode(args.arg_POSTGRES_URI.as_str(), DEFAULT_ENCODE_SET).to_string();
    match a_task {
        Some(task) => {
            let dracula_file = task
                .run(
                    utf8_percent_encode(args.arg_POSTGRES_URI.as_str(), DEFAULT_ENCODE_SET)
                        .to_string()
                        .as_str(),
                )
                .await;
            let path = format!("{}/{}/{}.parquet", MAIN_FOLDER, file_name, file_name); //=> purchase_orders/purchase_orders.parquet
            println!("{:#?} file", &dracula_file);
            let dracula_file_for_metadata = &dracula_file.0;
            let rows_len = &dracula_file.1;
            let _dracula_file_for_message = dracula_file.clone();
            let metadata = fs::metadata(dracula_file_for_metadata).unwrap();
            println!("{:#?} meta", metadata);
            println!("{:#?} rows_len", rows_len);
            println!("{:#?} meta bytes", metadata.len());

            let file1 = File::open(Path::new(&dracula_file.0)).unwrap();
            let reader = SerializedFileReader::new(file1).unwrap();
            let iter = reader.get_row_iter(None).unwrap();
            println!("{:?} lines", iter.count());
            info!("file {} size ", convert(metadata.len() as f64));
            multipart_upload(dracula_file.0.into(), BASE_PATH, path.as_str())
                .await
                .expect("upload file to s3");
            info!("file {} has been uploaded", file_name);
            let s3_path = format!("{}/{}/{}", BASE_PATH, MAIN_FOLDER, file_name);
            update_crawler(DATABASE.to_string(), CRAWLER_NAME_ONE.to_string(), s3_path)
                .await
                .expect("update crawler");
            start_crawler(CRAWLER_NAME_ONE.to_string(), true)
                .await
                .expect("start crawler");
        }
        _ => {
            error!("sorry, I don't know the task {}", args.flag_table)
        }
    }
}

async fn streaming_tasks() -> (f64, i32) {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());
    let pgi = &utf8_percent_encode(args.arg_POSTGRES_URI.as_str(), DEFAULT_ENCODE_SET).to_string();

    let tasks = streaming_tasks_list();

    let upload_size = AtomicUsize::new(0);
    let count = AtomicUsize::new(0);
    for (name, task) in tasks {
        println!("{:?} task ", name);
        let result: Result<(), Error> = {
            let _dracula_result = task.run(pgi).await; //- one tasks going
                                                       // let dracula_time = Instant::now();
            let path = format!("{}/{}/{}.parquet", MAIN_FOLDER, name, name);
            let dracula_file = format!("/tmp/{}.parquet", name);

            println!("{:?} file", &dracula_file);
            let dracula_file_for_metadata = &dracula_file;
            // let rows_len = &dracula_result.1;
            count.store(count.load(SeqCst) + 1, SeqCst);
            let metadata = fs::metadata(dracula_file_for_metadata).unwrap();
            upload_size.store(upload_size.load(SeqCst) + metadata.len() as usize, SeqCst);

            trace!("file {} size ", metadata.len());
            trace!("dracula file {}", dracula_file);
            upload(dracula_file.into(), BASE_PATH, path.as_str())
                .await
                .expect("upload file to s3 from all - must work");
            // let dracula_time = dracula_time.elapsed().as_millis();
            // let file = format!("{}.parquet", name);
            Ok(())
        };

        match result {
            Ok(()) => continue,
            error => {
                let message = format!("Dracula GOT Panic! in {} file", name);
                info!("{} {:?}", message, error);
                sentry::capture_message(message.as_str(), sentry::Level::Warning);
                info!("had an error while working on {}", name);
            }
        }
    }
    println!("all of them");
    (upload_size.load(SeqCst) as f64, count.load(SeqCst) as i32)
}

async fn ads_streaming_tasks() -> (f64, i32) {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());
    let pgi = &utf8_percent_encode(args.arg_POSTGRES_URI.as_str(), DEFAULT_ENCODE_SET).to_string();

    let tasks = ads_streaming_tasks_list();

    let upload_size = AtomicUsize::new(0);
    let count = AtomicUsize::new(0);
    for (name, task) in tasks {
        println!("{:?} task ", name);
        let result: Result<(), Error> = {
            let _dracula_result = task.run(pgi).await; //- one tasks going
                                                       // let dracula_time = Instant::now();

            let path = format!("{}/ads/{}/{}.parquet", MAIN_FOLDER, name, name)
                .replace("_stream", "")
                .replace("_task", "");
            println!("{:?} path", path);
            let dracula_file =
                format!("/tmp/{}.parquet", name.replace("_stream", "")).replace("_task", "");

            println!("{:?} file", &dracula_file);
            let dracula_file_for_metadata = &dracula_file;
            // let rows_len = &dracula_result.1;
            count.store(count.load(SeqCst) + 1, SeqCst);
            let metadata = fs::metadata(dracula_file_for_metadata).unwrap();
            upload_size.store(upload_size.load(SeqCst) + metadata.len() as usize, SeqCst);

            trace!("file {} size ", metadata.len());
            trace!("dracula file {}", dracula_file);
            upload(dracula_file.into(), BASE_PATH, path.as_str())
                .await
                .expect("upload file to s3 from all - must work");
            use glob::glob;
            for entry in glob("/tmp/*.parquet").expect("Failed to read glob pattern") {
                match entry {
                    Ok(path) => fs::remove_file(path).unwrap(),
                    Err(e) => println!("{:?}", e),
                }
            }
            Ok(())
        };

        match result {
            Ok(()) => continue,
            error => {
                let message = format!("Dracula GOT Panic! in {} file", name);
                info!("{} {:?}", message, error);
                sentry::capture_message(message.as_str(), sentry::Level::Warning);
                info!("had an error while working on {}", name);
            }
        }
    }
    println!("all of them");
    (upload_size.load(SeqCst) as f64, count.load(SeqCst) as i32)
}

#[tokio::main]
async fn main() {
    // Required to make static musl builds happy
    openssl_probe::init_ssl_cert_env_vars();

    pretty_env_logger::init();

    let dracula_time = Instant::now();
    let mut labels = HashMap::new();
    // #[cfg(not(debug_assertions))]
    let _guard = sentry::init((
        "https://id@id2.ingest.sentry.io/id4?timeout=10,verify_ssl=0",
        sentry::ClientOptions {
            release: sentry::release_name!(),
            environment: Some("production".into()),
            ..Default::default()
        },
    ));

    ::rayon::ThreadPoolBuilder::new()
        .num_threads(2)
        .build_global()
        .unwrap();

    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    let _current_timestamp = chrono::offset::Utc::now();

    if args.flag_file == "booster" {
        booster_command().await
    } else if args.flag_ads == "ads" {
        ads_clicks().await;
    } else if args.flag_table == "all" {
        let (_size, _count) = all(args.flag_table);

        let s3_path = format!("{}/{}", BASE_PATH, MAIN_FOLDER);
        create_crawler(CRAWLER_NAME.to_string(), s3_path, true)
            .await
            .expect("create crawler");
        start_crawler(CRAWLER_NAME.to_string(), true)
            .await
            .expect("start crawler");

        let dracula_time = dracula_time.elapsed().as_millis();
        info!("{} dracula time", dracula_time);

        // batch_size += count as i64;

        labels.insert("batch_name".to_string(), "all".to_string());
    } else if args.flag_table == "events" {
        let (_size, _count) = events().await;
        // batch_size += count as i64;
        // batch_size += count;
        labels.insert("batch_name".to_string(), "events".to_string());
    } else if args.flag_table == "events_emails" {
        let (_size, _count) = events_emails().await;
        // batch_size += count as i64;
        // batch_size += count;
        labels.insert("batch_name".to_string(), "events_emails_forms".to_string());
    } else if args.flag_table == "embedded_dynamic_forms" {
        let (_size, _count) = embedded_dynamic_forms().await;
        // batch_size += count as i64;
        // batch_size += count;
        labels.insert(
            "batch_name".to_string(),
            "embedded_dynamic_forms".to_string(),
        );
    } else if args.flag_table == "inventories" {
        inventories_task().await;
        labels.insert("batch_name".to_string(), "single_task".to_string());
    } else if args.flag_table == "streaming_tasks" {
        streaming_tasks().await;
        println!("after streaming_tasks().await ");
        let s3_path = format!("{}/{}", BASE_PATH, MAIN_FOLDER);
        create_crawler(CRAWLER_NAME.to_string(), s3_path.clone(), true)
            .await
            .expect("create crawler");
        update_crawler(DATABASE.to_string(), CRAWLER_NAME_ONE.to_string(), s3_path)
            .await
            .expect("update crawler");
        start_crawler(CRAWLER_NAME.to_string(), true)
            .await
            .expect("start crawler");

        let dracula_time = dracula_time.elapsed().as_millis();
        info!("{} dracula time", dracula_time);

        // batch_size += 1_i64;

        labels.insert("batch_name".to_string(), "streaming_tasks".to_string());
    } else if args.flag_table == "ads_streaming" {
        ads_streaming_tasks().await;
        println!("after ads streaming_tasks().await ");
        let s3_path = format!("{}/{}/{}/", BASE_PATH, MAIN_FOLDER, ADS_FOLDER);
        create_crawler(ADS_CRAWLER.to_string(), s3_path.clone(), true)
            .await
            .expect("create crawler");
        update_crawler(ADS_DATABASE.to_string(), ADS_CRAWLER.to_string(), s3_path)
            .await
            .expect("update crawler");
        start_crawler(ADS_CRAWLER.to_string(), true)
            .await
            .expect("start crawler");

        let dracula_time = dracula_time.elapsed().as_millis();
        info!("{} dracula time", dracula_time);

        // batch_size += 1_i64;

        labels.insert("batch_name".to_string(), "streaming_tasks".to_string());
    } else {
        single_task().await;
        labels.insert("batch_name".to_string(), "single_task".to_string());
    }
}

#[cfg(test)]
mod tests {
    use dracula_cli::*;

    #[test]
    fn test_task_lists_sizes() {
        assert_eq!(event_tasks_list().len(), 1);
        // assert_eq!(tasks_stream_list().len(), 1);
    }
}
