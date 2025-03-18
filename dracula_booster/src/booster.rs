extern crate openssl;
use crate::diesel::RunQueryDsl;
use bigdecimal::{BigDecimal, Zero};
use diesel::connection::SimpleConnection;
use diesel::{table, Connection, Insertable, PgConnection, Queryable};
use dracula_aws::aws::{download, get_execution_id};
use serde::Deserialize;
use tokio::time::Instant;
#[derive(Queryable, Debug, Default, Deserialize)]
#[allow(dead_code)]
pub struct Booster {
    pub id: i64,
    pub organization_id: i32,
    pub ware_id: i32,
    pub provider_id: i32,
    pub booster: f64,
}

pub async fn reading_boost() -> Vec<Booster> {
    let query = "select cast(id as integer) id, cast(organization_id as integer) organization_id, cast(ware_id as integer) ware_id,  cast(provider_id as integer) provider_id, updated_booster as booster  from \
    updated_booster  where update = 'true'";
    let catalog = Some("AwsDataCatalog".to_string());
    let database = Some("datascience_parquet".to_string());
    let bucket_query = "aws-athena-query-results-id-region-1";

    let key = format!(
        "{}.csv",
        get_execution_id(query.to_string(), catalog, database).await
    );
    println!("{:?} key", &key);

    let reader_body = match download(bucket_query, &key).await {
        Ok(body) => Some(body),
        Err(error) => {
            println!("{:?} error", error);
            None
        }
    };
    let read_unwrap = reader_body.unwrap();
    let mut reader = csv::Reader::from_reader(&*read_unwrap);
    let mut boost_data: Vec<Booster> = vec![Booster {
        ..Default::default()
    }];
    for record in reader.deserialize() {
        let record: Booster = record.unwrap();
        boost_data.push(Booster {
            id: record.id,
            organization_id: record.organization_id,
            ware_id: record.ware_id,
            provider_id: record.provider_id,
            booster: record.booster,
        });
    }
    println!("{:?} booster len", boost_data.len());

    boost_data
}

table! {
    temp_wpcs (id) {
        id -> Int8,
        organization_id -> Int4,
        ware_id -> Int4,
        provider_id -> Int4,
        booster -> Numeric,
        }
}

#[derive(Queryable, Debug, Insertable)]
pub struct TempWpc {
    pub id: i64,
    pub organization_id: i32,
    pub ware_id: i32,
    pub provider_id: i32,
    pub booster: BigDecimal,
}

pub async fn insert_postgres(pg_uri: &str) {
    let boost_call_time = Instant::now();
    let mut boost_data = reading_boost().await;
    let _booster: Booster = boost_data.remove(0);
    println!(
        "{:?} boost_data time seconds",
        boost_call_time.elapsed().as_millis() / 1000
    );

    let conn = PgConnection::establish(pg_uri);

    match conn {
        Ok(ref _conn) => {
            println!("connected  ");
        }
        Err(e) => {
            println!("Connection error: {}", e);
            return;
        }
    };
    let conn = conn.unwrap();

    let boost_create_time = Instant::now();
    use crate::booster::temp_wpcs::dsl::temp_wpcs;
    let drop_query = "DROP TABLE IF EXISTS temp_wpcs";
    conn.execute(drop_query).unwrap();
    // !!! diesel macro create table
    let create_query = "CREATE TEMP TABLE temp_wpcs (id BIGINT NOT NULL, ware_id INT NOT NULL, provider_id INT NOT NULL,
               organization_id INT NOT NULL, booster NUMERIC(3, 2) NOT NULL,
               CONSTRAINT temp_wpcs_pkey PRIMARY KEY (ware_id, provider_id, organization_id ))";

    conn.execute(create_query).unwrap();
    println!(" table created ");

    let chunk_len = 13_000;
    let mut count = 0;
    let mut end = 13_000;
    let total = boost_data.len();
    if end > total {
        end = total
    }
    while count < total {
        let mut temp_wpc: Vec<TempWpc> = vec![TempWpc {
            id: 0,
            organization_id: 0,
            ware_id: 0,
            provider_id: 0,
            booster: BigDecimal::zero(),
        }];
        let _temp_wpc_vector: TempWpc = temp_wpc.remove(0);
        for record in &boost_data[count..end] {
            temp_wpc.push(TempWpc {
                id: record.id,
                organization_id: record.organization_id,
                ware_id: record.ware_id,
                provider_id: record.provider_id,
                booster: bigdecimal::BigDecimal::from(record.booster),
            });
        }

        conn.transaction::<_, diesel::result::Error, _>(|| {
            diesel::insert_into(temp_wpcs)
                .values(&temp_wpc)
                .execute(&conn)
        })
        .unwrap();

        count += chunk_len;

        end += chunk_len;
        if end > total {
            end = total
        }
    }
    println!(" inserted ");
    println!(
        "{:?} boost insert time sec ",
        boost_create_time.elapsed().as_millis() / 1_000
    );

    let boost_update_time = Instant::now();

    conn.transaction::<_, diesel::result::Error, _>(|| {
        conn.batch_execute(
            "UPDATE ware_provider_configurations a \
                    SET booster = b.booster \
                    FROM temp_wpcs b \
                    WHERE a.ware_id=b.ware_id and \
                          a.organization_id=b.organization_id and \
                          a.provider_id=b.provider_id; \
                DROP TABLE IF EXISTS temp_wpcs;",
        )
    })
    .unwrap();

    println!(
        "{:?} boost update time sec ",
        boost_update_time.elapsed().as_millis() / 1_000
    );
}
