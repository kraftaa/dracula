use super::prelude::*;
use dracula_schemas::tables::shippings::dsl as shippings_dsl;

#[derive(ParquetRecordWriter)]
struct ShippingRecord {
    id: i32,
    shipable_id: Option<i32>,
    shipable_type: Option<String>,
    cost: f64,
    free_shipping: String,
    currency: Option<String>,
    notes: Option<String>,
    created_at: Option<NaiveDateTime>,
    updated_at: Option<NaiveDateTime>,
}

pub fn shippings(pg_uri: &str) -> (String, i64) {
    let conn = PgConnection::establish(pg_uri).unwrap();

    let shippings_load = Instant::now();
    let shippings = shippings_dsl::shippings.load::<Shipping>(&conn).unwrap();
    trace!("shippings: {:?}", shippings_load.elapsed());

    let path = "/tmp/shippings.parquet";

    let records: Vec<ShippingRecord> = shippings
        .iter()
        .map(|d| {

            ShippingRecord {
                id: d.id,
                shipable_id: d.shipable_id,
                shipable_type: d.shipable_type.clone(),
                cost: d.cost.to_f64().expect("big decimal price"),
                free_shipping: d.free_shipping.to_string(),
                currency: d.currency.clone(),
                notes: d.notes.clone(),
                created_at: d.created_at,
                updated_at: d.updated_at,
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

pub struct ShippingTask {}

impl DraculaTask for ShippingTask {
    fn run(&self, postgres_uri: &str) -> (String, i64) {
        shippings(postgres_uri)
    }
}
