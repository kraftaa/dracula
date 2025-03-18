use crate::ads_data::clicks::dsl::clicks;
use crate::ads_data::{last_folders_to_load, load_object_from_s3, Click};
use aws_sdk_s3::Client;
use chrono::Datelike;
use diesel::connection::SimpleConnection;
use diesel::{Connection, PgConnection, RunQueryDsl};
use dracula_aws::aws::paginagion_stream_code;
use serde_json::Value;

pub async fn insert_ads_clicks(pg_uri: &str) -> eyre::Result<()> {
    let sdk_config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let client = Client::new(&sdk_config);
    let bucket = "domain-ads-production";
    let conn = PgConnection::establish(pg_uri);

    match conn {
        Ok(ref _conn) => {
            println!("connected  ");
        }
        Err(e) => {
            println!("Connection error: {}", e);
            return Err(eyre::Report::new(e));
        }
    };
    let conn = conn.unwrap();
    println!("create clicks");
    match conn.batch_execute(
        "
        CREATE TABLE IF NOT EXISTS clicks
    (
        id VARCHAR NOT NULL,
        data JSONB,
        file_name VARCHAR,
        last_modified_date timestamp,
        year int,
        PRIMARY KEY (id, year)
    ) PARTITION BY RANGE (year);
        CREATE TABLE IF NOT EXISTS clicks_2021 PARTITION OF clicks
        FOR VALUES FROM (2021) TO (2022);
        CREATE TABLE IF NOT EXISTS clicks_2022 PARTITION OF clicks
        FOR VALUES FROM (2022) TO (2023);
        CREATE TABLE IF NOT EXISTS clicks_2023 PARTITION OF clicks
        FOR VALUES FROM (2023) TO (2024);
        CREATE TABLE IF NOT EXISTS clicks_2024 PARTITION OF clicks
        FOR VALUES FROM (2024) TO (2025);
        CREATE TABLE IF NOT EXISTS clicks_2025 PARTITION OF clicks
        FOR VALUES FROM (2025) TO (2026);

    ",
    ) {
        Ok(_) => {}
        Err(e) => eprintln!("Error creating table clicks: {:?}", e),
    };
    let last_folders_to_load = last_folders_to_load(true);
    for folder in &last_folders_to_load {
        let prefix = folder.to_string();
        println!("folder {}", &prefix);
        let mut object_stream = paginagion_stream_code(&client, bucket, &prefix).await;

        while let Ok(Some(chunk)) = object_stream.try_next().await {
            for object in chunk.contents.unwrap() {
                let client_clone = client.clone();
                let object_key = object.clone().key.unwrap_or_default();
                if object_key.contains("clicks") {
                    let load_task = async move {
                        load_object_from_s3((object, client_clone)).await // Use the cloned client
                    };
                    let (body, time, key) = load_task.await;

                    let mut value_clicks: Vec<Click> = vec![Click {
                        id: "".to_string(),
                        data: None,
                        file_name: None,
                        last_modified_date: None,
                        year: 0,
                    }];
                    let _value_clicks: Click = value_clicks.remove(0);

                    for line in body.lines() {
                        let v: Value = serde_json::from_str(line).unwrap();
                        let id = v["Id"].to_string();
                        let file_name = Some(key.to_string());
                        let data: Option<Value> = Some(v);
                        value_clicks.push(Click {
                            id,
                            data,
                            file_name,
                            last_modified_date: Some(time),
                            year: time.year(),
                        });
                    }
                    // Postgres insert limit is 65_000 cells
                    let chunk_size = 13_000;
                    let value_clicks_in_chunks = value_clicks.chunks(chunk_size);

                    for chunk_value in value_clicks_in_chunks {
                        {
                            conn.transaction::<_, diesel::result::Error, _>(|| {
                                diesel::insert_into(clicks)
                                    .values(chunk_value)
                                    .on_conflict_do_nothing()
                                    .execute(&conn)
                            })
                            .expect("Failed to insert into clicks");
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
