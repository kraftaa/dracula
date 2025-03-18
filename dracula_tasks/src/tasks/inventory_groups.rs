use super::prelude::*;
use dracula_schemas::tables::inventory_groups::dsl as inventory_groups_dsl;
// use uuid::Uuid;

#[derive(ParquetRecordWriter)]
struct InventoryGroupRecord {
    id: i32,
    index: i32,
    metadata: Option<String>,
    shipping_cost: f64,
    created_at: Option<NaiveDateTime>,
    updated_at: Option<NaiveDateTime>,
    uuid: Option<String>,
    deleted_at: Option<NaiveDateTime>,
}

pub fn inventory_groups(pg_uri: &str) -> (String, i64) {
    let conn = PgConnection::establish(pg_uri).unwrap();

    let inventory_groups_load = Instant::now();
    let inventory_groups = inventory_groups_dsl::inventory_groups
        .load::<InventoryGroup>(&conn)
        .unwrap();
    trace!("inventory_groups: {:?}", inventory_groups_load.elapsed());

    let path = "/tmp/inventory_groups.parquet";

    let records: Vec<InventoryGroupRecord> = inventory_groups
        .iter()
        .map(|ig| {
            let metadata = ig.metadata.as_ref().map(|h| h.clone().to_string());

            InventoryGroupRecord {
                id: ig.id,
                index: ig.index,
                metadata,
                shipping_cost: ig
                    .shipping_cost
                    .to_f64()
                    .expect("big decimal shipping cost"),
                created_at: ig.created_at,
                updated_at: ig.updated_at,
                uuid: Some(ig.uuid.unwrap().to_string()),
                deleted_at: ig.deleted_at,
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

pub struct InventoryGroupTask {}

impl DraculaTask for InventoryGroupTask {
    fn run(&self, postgres_uri: &str) -> (String, i64) {
        inventory_groups(postgres_uri)
    }
}
