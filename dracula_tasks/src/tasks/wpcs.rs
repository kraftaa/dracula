use super::prelude::*;
pub use futures_util::stream::StreamExt;
use parquet::record::RecordWriter;
pub use sqlx::postgres::PgPool;

#[derive(ParquetRecordWriter, Default, sqlx::FromRow, Debug)]
struct WPCStreamRecord {
    id: i64,
    ware_id: Option<i32>,
    provider_id: Option<i32>,
    required: Option<bool>,
    created_at: Option<NaiveDateTime>,
    updated_at: Option<NaiveDateTime>,
    organization_id: Option<i32>,
    auto_assigned: Option<bool>,
    booster: f64,
}

pub async fn wpcs(pg_uri: &str) -> anyhow::Result<(String, i64)> {
    let pool = PgPool::connect(pg_uri).await?;

    let fake_wpc = vec![WPCStreamRecord {
        ..Default::default()
    }];

    let vector_for_schema = &fake_wpc;
    let schema = vector_for_schema.as_slice().schema().unwrap();
    let schema_2 = vector_for_schema.as_slice().schema().unwrap();
    let schema_vec = schema_2.get_fields();

    let mut fields: Vec<&str> = vec![];
    for i in schema_vec {
        if i.name() == "booster" {
            fields.push("booster::float")
        } else {
            fields.push(i.name())
        }
    }
    println!("{:?} fields!", fields);

    println!("{:?} schema", &schema);
    let path = "/tmp/wpcs.parquet";
    let path_meta = <&str>::clone(&path);
    let file = std::fs::File::create(path).unwrap();
    let mut pfile = SerializedFileWriter::new(file, schema, props()).unwrap();
    let table: &str = "wpcs";

    let mut query = "SELECT ".to_owned();
    let fields: &str = &fields.join(", ");
    query.push_str(fields);
    query.push_str(" FROM ");
    query.push_str(table);

    let q = sqlx::query_as::<sqlx::Postgres, WPCStreamRecord>(&query);

    let wpc_stream = q.fetch(&pool);
    println!("{} query", query);

    let mut chunk_stream = wpc_stream.map(|fs| fs.unwrap()).chunks(5000);
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
    let file_metadata = parquet_metadata.file_metadata();
    let rows_number = file_metadata.num_rows();
    Ok((path.into(), rows_number))
}

use async_trait::async_trait;
#[derive(Debug)]
pub struct WPCStreamingTask {}
#[async_trait]
impl DraculaStreamingTask for WPCStreamingTask {
    async fn run(&self, postgres_uri: &str) -> (String, i64) {
        wpcs(postgres_uri).await.unwrap()
    }
}
