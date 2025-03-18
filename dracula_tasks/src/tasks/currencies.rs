use super::prelude::*;

use dracula_schemas::tables::currencies::dsl as currencies_dsl;

#[derive(ParquetRecordWriter)]
struct CurrencyRecord {
    id: i32,
    exchangable_id: Option<i32>,
    exchangable_type: Option<String>,
    currency: Option<String>,
    conversion_rate: f64,
    conversion_set_at: Option<NaiveDateTime>,
    conversion_history: Option<String>,
    created_at: Option<NaiveDateTime>,
    updated_at: Option<NaiveDateTime>,
}

pub fn currencies(pg_uri: &str) -> (String, i64) {
    let conn = PgConnection::establish(pg_uri).unwrap();

    let currencies_load = Instant::now();
    let currencies = currencies_dsl::currencies.load::<Currency>(&conn).unwrap();
    trace!("load currencies took: {:?}", currencies_load.elapsed());

    let path = "/tmp/currencies.parquet";

    let records: Vec<CurrencyRecord> = currencies
        .iter()
        .map(|c| {
            let conversion_history = c.conversion_history.as_ref().map(|h| h.clone().to_string());

            CurrencyRecord {
                id: c.id,
                exchangable_id: c.exchangable_id,
                exchangable_type: c.exchangable_type.clone(),
                currency: c.currency.clone(),
                conversion_rate: c
                    .conversion_rate
                    .as_ref()
                    .unwrap()
                    .to_f64()
                    .expect("big decimal rate"),
                conversion_set_at: c.conversion_set_at,
                conversion_history,
                // origin: c.origin.clone(),
                // access,
                created_at: c.created_at,
                updated_at: c.updated_at,
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

pub struct CurrencyTask {}

impl DraculaTask for CurrencyTask {
    fn run(&self, postgres_uri: &str) -> (String, i64) {
        currencies(postgres_uri)
    }
}
