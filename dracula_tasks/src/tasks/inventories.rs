use super::prelude::*;

pub use futures_util::stream::StreamExt;
pub use sqlx::postgres::PgPool;

#[derive(ParquetRecordWriter, Default, sqlx::FromRow, Debug)]
struct InventoriesStream {
    id: i64,
    inventory_group_id: Option<i32>,
    data: Option<String>,
    created_at: Option<NaiveDateTime>,
    updated_at: Option<NaiveDateTime>,
    uuid: Option<String>,
    product_name: Option<String>,
    product_url: Option<String>,
    provider_id: Option<i32>,
}

pub async fn inventories(pg_uri: &str) -> anyhow::Result<(String, i64)> {
    let pool = PgPool::connect(pg_uri).await?;
    let fake_inventory_items = vec![InventoriesStream {
        ..Default::default()
    }];

    let vector_for_schema = &fake_inventory_items;
    let schema = vector_for_schema.as_slice().schema().unwrap();
    let schema_2 = vector_for_schema.as_slice().schema().unwrap();
    let schema_vec = schema_2.get_fields();
    //
    let mut fields: Vec<&str> = vec![];
    for i in schema_vec {
        if i.name() == "uuid" {
            fields.push("uuid::varchar")
        } else {
            fields.push(i.name())
        }
    }
    println!("{:?} fields!", fields);
    let path = "/tmp/inventories.parquet";
    let path_meta = <&str>::clone(&path);

    let file = std::fs::File::create(path).unwrap();
    let mut pfile = SerializedFileWriter::new(file, schema, props()).unwrap();

    let query = "select id, inventory_group_id, data::varchar, created_at, updated_at, \
     product_name, product_url, provider_id,\
     uuid::varchar from inventories";

    let q = sqlx::query_as::<sqlx::Postgres, InventoriesStream>(query);

    let pr_stream = q.fetch(&pool);
    println!("{} query", query);

    let mut chunk_stream = pr_stream.map(|fs| fs.unwrap()).chunks(5000);

    while let Some(chunks) = chunk_stream.next().await {
        // println!("{:?} data", &chunks);
        let mut row_group = pfile.next_row_group().unwrap();
        (&chunks[..])
            .write_to_row_group(&mut row_group)
            .expect("can't 'write_to_row_group' ...");
        pfile.close_row_group(row_group).unwrap();
    }

    // let rows_number = *pfile.total_num_rows() as i64;
    pfile.close().unwrap();
    let reader = SerializedFileReader::try_from(path_meta).unwrap();
    let parquet_metadata = reader.metadata();
    let file_metadata = parquet_metadata.file_metadata();
    let rows_number = file_metadata.num_rows();
    println!("{:?} rows", rows_number);
    Ok((path.into(), rows_number))
}

use async_trait::async_trait;
#[derive(Debug)]
pub struct InventoriesStTask {}
#[async_trait]
impl DraculaStreamingTask for InventoriesStTask {
    async fn run(&self, postgres_uri: &str) -> (String, i64) {
        inventories(postgres_uri).await.unwrap()
    }
}
