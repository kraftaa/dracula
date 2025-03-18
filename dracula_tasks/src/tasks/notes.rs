use super::prelude::*;
pub use futures_util::stream::StreamExt;
use parquet::record::RecordWriter;
pub use sqlx::postgres::PgPool;

#[derive(ParquetRecordWriter, Default, sqlx::FromRow, Debug)]
struct NotesRecordStream {
    id: i64,
    title: Option<String>,
    body: Option<String>,
    status: Option<String>,
    request_id: Option<i32>,
    ware_id: Option<String>,
    uuid: String,
    user_id: Option<i32>,
    invoice_id: Option<i32>,
    purchase_order_id: Option<i32>,
    proposal_id: Option<i32>,
    created_at: Option<NaiveDateTime>,
    updated_at: Option<NaiveDateTime>,
}

pub async fn notes(pg_uri: &str) -> anyhow::Result<(String, i64)> {
    let pool = PgPool::connect(pg_uri).await?;
    let fake_ng = vec![NotesRecordStream {
        ..Default::default()
    }];

    let vector_for_schema = &fake_ng;
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

    let notes_load = Instant::now();
    let path = "/tmp/notes_pg.parquet";
    let path_meta = <&str>::clone(&path);

    // let props = Arc::new(WriterProperties::builder().build());
    let file = std::fs::File::create(path).unwrap();
    let mut pfile = SerializedFileWriter::new(file, schema, props()).unwrap();
    let table: &str = "notes";

    let mut query = "SELECT ".to_owned();
    let fields: &str = &fields.join(", ");
    query.push_str(fields);
    query.push_str(" FROM ");
    query.push_str(table);

    let q = sqlx::query_as::<sqlx::Postgres, NotesRecordStream>(&query);

    let notes_stream = q.fetch(&pool);
    println!("{} query", query);
    println!(" before stream");
    trace!("load notes took: {:?}", notes_load.elapsed());

    let mut chunk_stream = notes_stream.map(|fs| fs.unwrap()).chunks(5000);
    while let Some(chunks) = chunk_stream.next().await {
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
    println!("{:?} num row group", parquet_metadata.num_row_groups());

    let file_metadata = parquet_metadata.file_metadata();
    let rows_number = file_metadata.num_rows();
    println!("{:?} file_metadata.num_rows()", file_metadata.num_rows());
    Ok((path.into(), rows_number))
}

use async_trait::async_trait;
#[derive(Debug)]
pub struct NoteStreamingTask {}
#[async_trait]
impl DraculaStreamingTask for NoteStreamingTask {
    async fn run(&self, postgres_uri: &str) -> (String, i64) {
        notes(postgres_uri).await.unwrap()
    }
}
