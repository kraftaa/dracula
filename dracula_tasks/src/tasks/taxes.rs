use super::prelude::*;

use std::str;

use dracula_schemas::tables::taxes::dsl as taxes_dsl;

#[derive(ParquetRecordWriter)]
struct TaxesRecord {
    id: i32,
    taxable_id: Option<i32>,
    taxable_type: Option<String>,
    amount: f64,
    category: Option<String>,
    currency: Option<String>,
    description: Option<String>,
    locale: Option<String>,
    notes: Option<String>,
    percent_rate: f64,
    created_at: Option<NaiveDateTime>,
    updated_at: Option<NaiveDateTime>,
    type_: Option<String>,
}

pub fn taxes(pg_uri: &str) -> (String, i64) {
    let pgconn = PgConnection::establish(pg_uri).unwrap();

    let tax_load = Instant::now();
    let taxes = taxes_dsl::taxes.load::<Tax>(&pgconn).unwrap();
    trace!("loading taxes took: {:?}", tax_load.elapsed());

    let path = "/tmp/taxes.parquet";

    let records: Vec<TaxesRecord> = taxes
        .iter()
        .map(|t| {

            TaxesRecord {
                id: t.id,
                taxable_id: t.taxable_id,
                taxable_type: t.taxable_type.clone(),
                amount: t.amount.to_f64().unwrap(),
                category: t.category.clone(),
                currency: t.currency.clone(),
                description: t.description.clone(),
                locale: t.locale.clone(),
                notes: t.notes.clone(),
                percent_rate: t.percent_rate.to_f64().unwrap(),
                created_at: t.created_at,
                updated_at: t.updated_at,
                type_: t.type_.clone(),
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

pub struct TaxTask {}

impl DraculaTask for TaxTask {
    fn run(&self, postgres_uri: &str) -> (String, i64) {
        taxes(postgres_uri)
    }
}
