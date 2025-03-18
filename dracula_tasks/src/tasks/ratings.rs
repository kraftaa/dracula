use super::prelude::*;

use dracula_schemas::tables::{providers::dsl as providers_dsl, ratings::dsl as ratings_dsl};

#[derive(ParquetRecordWriter)]
struct RatingShort {
    id: i32,
    value: i32,
    ware_id: Option<String>,
    spam: String,
    provider_id: Option<i32>,
    provider_name: Option<String>,
    created_at: Option<NaiveDateTime>,
    created_on: Option<NaiveDate>,
    request_id: Option<i32>,
    order_id: Option<i32>,
    comment: Option<String>,
    organization_id: Option<i32>,
    user_id: Option<i32>,
}

pub fn ratings(pg_uri: &str) -> (String, i64) {
    let conn = PgConnection::establish(pg_uri).unwrap();

    let ratings_load = Instant::now();
    let ratings = ratings_dsl::ratings.load::<Rating>(&conn).unwrap();
    trace!("load ratings took: {:?}", ratings_load.elapsed());

    let provider_ids: Vec<_> = ratings.iter().filter_map(|x| x.provider_id).collect();
    let providers_load = Instant::now();
    let providers: Vec<Provider> = providers_dsl::providers
        .filter(providers_dsl::id.eq(any(&provider_ids[..])))
        .load::<Provider>(&conn)
        .unwrap();
    trace!(
        "load providers ({}) took: {:?}",
        providers.len(),
        providers_load.elapsed()
    );

    let path = "/tmp/ratings.parquet";

    let records: Vec<RatingShort> = ratings
        .iter()
        .filter(|rt| !rt.spam)
        .map(|r| {
            let provider = providers.iter().find(|y| Some(y.id) == r.provider_id);
            let provider_name = if let Some(p) = provider {
                p.name.clone()
            } else {
                None
            };

            let spam = r.spam.to_string();
            // let access = r.access.clone().join(", ");

            RatingShort {
                id: r.id,
                value: r.value,
                ware_id: r.ware_id.clone(),
                spam,
                provider_id: r.provider_id,
                provider_name,
                created_at: r.created_at,
                created_on: r.created_at.map(|ca| ca.date()),
                request_id: r.request_id,
                order_id: r.order_id,
                comment: r.comment.clone(),
                organization_id: r.organization_id,
                user_id: r.user_id,
            }
        })
        .collect();

    let path_meta = <&str>::clone(&path);
    let vector_for_schema = &records;
    let schema = vector_for_schema.as_slice().schema().unwrap();
    println!("{:?} schema", &schema);

    let file = std::fs::File::create(path).unwrap();
    let mut pfile = SerializedFileWriter::new(file, schema, props()).unwrap();
    {
        let mut row_group = pfile.next_row_group().unwrap();
        (&records[..])
            .write_to_row_group(&mut row_group)
            .expect("can't 'write_to_row_group' ...");
        pfile.close_row_group(row_group).unwrap();
    }

    pfile.close().unwrap();
    let reader = SerializedFileReader::try_from(path_meta).unwrap();
    let parquet_metadata = reader.metadata();
    let file_metadata = parquet_metadata.file_metadata();
    let rows_number = file_metadata.num_rows();
    (path.into(), rows_number)
}

pub struct RatingTask {}

impl DraculaTask for RatingTask {
    fn run(&self, postgres_uri: &str) -> (String, i64) {
        ratings(postgres_uri)
    }
}
