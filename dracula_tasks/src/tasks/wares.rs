use super::prelude::*;

use dracula_schemas::tables::wares::dsl as w_dsl;

#[derive(ParquetRecordWriter)]

struct WareRecord {
    id: i32,
    name: Option<String>,
    created_at: Option<NaiveDateTime>,
    updated_at: Option<NaiveDateTime>,
    slug: Option<String>,
    description: Option<String>,
    ware_type: Option<String>,
}
pub fn wares(pg_uri: &str) -> (String, i64) {
    let conn = PgConnection::establish(pg_uri).unwrap();

    let wares_load = Instant::now();
    let wares = w_dsl::wares.load::<Ware>(&conn).unwrap();
    trace!("wares: {:?}", wares_load.elapsed());

    let path = "/tmp/wares.parquet";

    let records: Vec<WareRecord> = wares
        .iter()
        .map(|w| WareRecord {
            id: w.id,
            name: w.name.clone(),
            created_at: w.created_at,
            updated_at: w.updated_at,
            slug: w.slug.clone(),
            description: w.description.clone(),
            ware_type: w.ware_type.clone(),
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

pub struct WareTask {}

impl DraculaTask for WareTask {
    fn run(&self, postgres_uri: &str) -> (String, i64) {
        wares(postgres_uri)
    }
}
