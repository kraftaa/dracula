use super::prelude::*;
pub use futures_util::stream::StreamExt;
use parquet::record::RecordWriter;
pub use sqlx::postgres::PgPool;

#[derive(ParquetRecordWriter, Default, sqlx::FromRow, Debug)]
struct RefsProvStreamingRecord {
    id: i64,
    type_: Option<String>,
    reference_of_type: Option<String>,
    reference_to_type: Option<String>,
    reference_to_id: Option<i32>,
    reference_of_id: Option<i32>,
    created_at: Option<NaiveDateTime>,
    updated_at: Option<NaiveDateTime>,
    name: Option<String>,
    slug: Option<String>,
    score: Option<f64>,
    contract_status: Option<String>,
}

pub async fn refs_providers(pg_uri: &str) -> anyhow::Result<(String, i64)> {
    let pool = PgPool::connect(pg_uri).await?;
    let fake_refs_prov = vec![RefsProvStreamingRecord {
        ..Default::default()
    }];
    let vector_for_schema = &fake_refs_prov;

    let schema = vector_for_schema.as_slice().schema().unwrap();
    let schema_2 = vector_for_schema.as_slice().schema().unwrap();
    let schema_vec = schema_2.get_fields();
    let mut fields: Vec<&str> = vec![];

    for i in schema_vec {
        if i.name() == "type" {
            fields.push("type as type_")
        } else if i.name() == "name" {
            fields.push("(fields->'name')::varchar as name ")
        } else if i.name() == "type_" {
            fields.push("type as type_ ")
        } else if i.name() == "slug" {
            fields.push("(fields->'slug')::varchar as slug ")
        } else if i.name() == "score" {
            fields.push("(fields->'score'->> 0)::float as score ")
        } else if i.name() == "contract_status" {
            fields.push("(fields->'contract_status')::varchar as contract_status ")
        } else {
            fields.push(i.name())
        }
    }

    println!("{:?} fields!", fields);
    let refs_pr_load = Instant::now();
    let path = "/tmp/refs_providers.parquet";
    let path_meta = <&str>::clone(&path);

    let file = std::fs::File::create(path).unwrap();
    let mut pfile = SerializedFileWriter::new(file, schema, props()).unwrap();
    let table: &str = "refs";

    let mut query = "SELECT ".to_owned();
    let fields: &str = &fields.join(", ");
    query.push_str(fields);
    query.push_str(" FROM ");
    query.push_str(table);

    let q = sqlx::query_as::<sqlx::Postgres, RefsProvStreamingRecord>(&query);

    let refs_pr_stream = q.fetch(&pool);
    println!("{} query", query);
    println!(" before stream");
    trace!("load refs providers took: {:?}", refs_pr_load.elapsed());

    let mut chunk_stream = refs_pr_stream.map(|fs| fs.unwrap()).chunks(5000);
    while let Some(chunks) = chunk_stream.next().await {
        let mut row_group = pfile.next_row_group().unwrap();
        (&chunks[..])
            .write_to_row_group(&mut row_group)
            .expect("can't 'write_to_row_group' ...");
        pfile.close_row_group(row_group).unwrap();
    }
    pfile.close().unwrap();
    let reader = SerializedFileReader::try_from(path_meta).unwrap();
    let parquet_metadata = reader.metadata();
    println!("{:?} num row group", parquet_metadata.num_row_groups());

    let file_metadata = parquet_metadata.file_metadata();
    let rows_number = file_metadata.num_rows();
    println!("{:?} file_metadata.num_rows()", file_metadata.num_rows());
    Ok((path.into(), rows_number))
}

use async_trait::async_trait;
#[derive(Debug)]
pub struct RefsPrStreamingTask {}
#[async_trait]
impl DraculaStreamingTask for RefsPrStreamingTask {
    async fn run(&self, postgres_uri: &str) -> (String, i64) {
        refs_providers(postgres_uri).await.unwrap()
    }
}
