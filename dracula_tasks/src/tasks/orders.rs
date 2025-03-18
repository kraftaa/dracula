use super::prelude::*;
use std::collections::HashMap;
use std::str;

use dracula_schemas::tables::{
    orders::dsl as orders_dsl, providers::dsl as providers_dsl, requests::dsl as requests_dsl,
    timepoints::dsl as time_dsl, users::dsl as users_dsl,
};

#[derive(ParquetRecordWriter)]
struct OrderTaskRecord {
    id: i32,
    provider_id: Option<i32>,
    provider_name: Option<String>,
    request_id: Option<i32>,
    user_email: Option<String>,
    client_id: Option<i32>,
    organization_id: Option<i32>,
    obligations: Option<String>, // Option<serde_json::Value>,
    state: String,
    declined: bool,
    order_cancelled: bool,
    request_cancelled: bool,
    last_computed_status: Option<String>,
    uuid: String,
    created_at: Option<NaiveDateTime>,
    updated_at: Option<NaiveDateTime>,
    approval_date: Option<NaiveDate>,
    review_date: Option<NaiveDate>,
    submission_date: Option<NaiveDate>,
    completion_date: Option<NaiveDate>,
    booster: Option<f64>,
}

pub fn orders(pg_uri: &str) -> (String, i64) {
    let conn = PgConnection::establish(pg_uri).expect("postgres conn");

    let orders_load = Instant::now();
    let orders = orders_dsl::orders.load::<Order>(&conn).unwrap();

    trace!("load orders took: {:?}", orders_load.elapsed());

    let path = "/tmp/orders.parquet";
    let path_meta = <&str>::clone(&path);

    let records_load = Instant::now();
    let mut count = 0;

    let providers_load = Instant::now();
    let orders_ids: Vec<i32> = orders.iter().map(|x| x.id).collect();

    let request_ids: Vec<i32> = orders.iter().map(|x| x.request_id.unwrap()).collect();
    let provider_ids: Vec<i32> = orders.iter().map(|x| x.provider_id.unwrap_or(0)).collect();

    let providers = providers_dsl::providers
        .filter(providers_dsl::id.eq(any(&provider_ids[..])))
        .load::<Provider>(&conn)
        .unwrap();
    trace!("load providers took: {:?}", providers_load.elapsed());

    let users_load = Instant::now();

    let users = users_dsl::users.load::<User>(&conn).unwrap();
    trace!("load users took: {:?}", users_load.elapsed());

    let requests_load = Instant::now();
    let requests = requests_dsl::requests
        .filter(requests_dsl::id.eq(any(&request_ids[..])))
        .load::<Request>(&conn)
        .unwrap();
    trace!("load QG_PG took: {:?}", requests_load.elapsed());

    let providers_by_provider_id: HashMap<i32, &Provider> =
        providers.iter().map(|x| (x.id, x)).collect();

    let users_by_user_id: HashMap<i32, &User> = users.iter().map(|x| (x.id, x)).collect();

    let requests_by_id: HashMap<i32, &Request> = requests.iter().map(|x| (x.id, x)).collect();

    let timepoints_load = Instant::now();
    let timepoints = time_dsl::timepoints
        .filter(time_dsl::order_id.eq(any(&orders_ids[..])))
        .load::<TimepointPG>(&conn)
        .unwrap();
    trace!("load timepoints took: {:?}", timepoints_load.elapsed());
    let timepoints_by_qw_id: HashMap<i32, &TimepointPG> =
        timepoints.iter().map(|x| (x.id, x)).collect();

    let records: Vec<OrderTaskRecord> = orders
        .par_iter()
        .map(|order| {
            let obligations = order.obligations.as_ref().map(|l| l.clone().to_string());

            let provider = providers_by_provider_id.get(&order.provider_id.unwrap_or(0));
            let provider_name = if let Some(p) = provider {
                p.name.clone()
            } else {
                None
            };

            let requests = requests_by_id.get(&order.request_id.unwrap_or(0));
            let organization_id = if let Some(qg) = requests {
                qg.organization_id
            } else {
                None
            };
            let user_id = if let Some(r) = requests {
                r.user_id
            } else {
                None
            };

            let user = users_by_user_id.get(&user_id.unwrap_or(0));
            let user_email = if let Some(u) = user {
                u.email.clone()
            } else {
                None
            };

            let timepoint = timepoints_by_qw_id.get(&order.id);

            let approval_date = if let Some(t) = timepoint {
                if t.name == Some("approval date".to_string()) {
                    t.date.map(|x| x.date())
                } else {
                    None
                }
            } else {
                None
            };

            let review_date = if let Some(t) = timepoint {
                if t.name == Some("review date".to_string()) {
                    t.date.map(|x| x.date())
                } else {
                    None
                }
            } else {
                None
            };

            let submission_date = if let Some(t) = timepoint {
                if t.name == Some("submission date".to_string()) {
                    t.date.map(|x| x.date())
                } else {
                    None
                }
            } else {
                None
            };

            let completion_date = if let Some(t) = timepoint {
                if t.name == Some("completion date".to_string()) {
                    t.date.map(|x| x.date())
                } else {
                    None
                }
            } else {
                None
            };

            let booster = order
                .booster
                .map(|cr| cr.to_f64().expect("bigdecimal to f64"))
                .expect("Unwrapping booster in Order");

            OrderTaskRecord {
                id: order.id,
                provider_id: order.provider_id,
                provider_name,
                request_id: order.request_id,
                user_email,
                client_id: order.client_id,
                organization_id,
                obligations,
                state: order.state.clone(),
                declined: order.declined,
                order_cancelled: order.order_cancelled,
                request_cancelled: order.request_cancelled,
                last_computed_status: order.last_computed_status.clone(),
                uuid: order.uuid.to_string(),
                created_at: order.created_at,
                updated_at: order.updated_at,
                approval_date,
                review_date,
                submission_date,
                completion_date,
                booster: Some(booster),
            }
        })
        .collect();

    let vector_for_schema = &records;
    let schema = vector_for_schema.as_slice().schema().unwrap();
    println!("{:?} schema", &schema);

    let file = std::fs::File::create(path).unwrap();
    let mut pfile = SerializedFileWriter::new(file, schema, props()).unwrap();

    trace!(
        "load QW_PG ({}) took: {:?}",
        records.len(),
        records_load.elapsed()
    );

    {
        let mut row_group = pfile.next_row_group().unwrap();
        (&records[..])
            .write_to_row_group(&mut row_group)
            .expect("can't 'write_to_row_group' ...");
        pfile.close_row_group(row_group).unwrap();
        count += 1;
        println!("{} count", count);
    }
    // }

    pfile.close().unwrap();
    let reader = SerializedFileReader::try_from(path_meta).unwrap();
    let parquet_metadata = reader.metadata();
    let file_metadata = parquet_metadata.file_metadata();
    let rows_number = file_metadata.num_rows();
    (path.into(), rows_number)
}

pub struct OrderTask {}

impl DraculaTask for OrderTask {
    fn run(&self, postgres_uri: &str) -> (String, i64) {
        orders(postgres_uri)
    }
}
