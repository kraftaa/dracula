use super::prelude::*;

use dracula_schemas::tables::milestones::dsl as milestones_dsl;

#[derive(ParquetRecordWriter)]
struct MilestoneRecord {
    id: i32,
    itemizable_id: Option<i32>,
    itemizable_type: Option<String>,
    _type: String,
    name: Option<String>,
    quantity: f64, // BigDecimal,
    // total_price: f64, // BigDecimal,
    state: String,
    tax_rate: f64, // BigDecimal,
    currency: Option<String>,
    unit_of_measure: Option<String>,
    classifications: String, //: serde_json::Value,
    line_number: Option<i32>,
    comments: Option<String>,
    created_at: Option<NaiveDateTime>,
    updated_at: Option<NaiveDateTime>,
    unit_price: f64,  // BigDecimal,
    total_price: f64, // BigDecimal,
    status: Option<String>,
    provider_id: Option<i32>,
    cancelled_at: Option<NaiveDateTime>,
    shipped_at: Option<NaiveDateTime>,
    estimated_date: Option<NaiveDate>,
}
fn date_at_or_placeholder(date: Option<NaiveDateTime>) -> NaiveDateTime {
    date.unwrap_or_else(|| {
        NaiveDateTime::new(
            NaiveDate::from_ymd_opt(1, 1, 1).unwrap(),
            chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
        )
    })
}

fn estimated_date_placeholder(date: Option<NaiveDate>) -> NaiveDate {
    date.unwrap_or_else(|| NaiveDate::from_ymd_opt(1, 1, 1).unwrap())
}

pub fn milestones(pg_uri: &str) -> (String, i64) {
    let conn = PgConnection::establish(pg_uri).unwrap();

    let milestones_load = Instant::now();
    let milestones = milestones_dsl::milestones.load::<Milestone>(&conn).unwrap();
    trace!("milestones: {:?}", milestones_load.elapsed());

    let path = "/tmp/milestones.parquet";

    let records: Vec<MilestoneRecord> = milestones
        .iter()
        .map(|m| {
            let classifications = m.classifications.clone().to_string();
            let unit_price = m.unit_price.to_f64().unwrap();
            let total_price = m.total_price.to_f64().unwrap();
            let quantity = m.quantity.to_f64().unwrap();
            let tax_rate = m.tax_rate.to_f64().unwrap();

            MilestoneRecord {
                id: m.id,
                itemizable_id: m.itemizable_id,
                itemizable_type: m.itemizable_type.clone(),
                unit_price,
                total_price,
                _type: m._type.clone(),
                name: m.name.clone(),
                quantity,
                state: m.state.clone(),
                tax_rate,
                currency: m.currency.clone(),
                unit_of_measure: m.unit_of_measure.clone(),
                classifications,
                line_number: m.line_number,
                comments: m.comments.clone(),
                created_at: Some(date_at_or_placeholder(m.created_at)),
                updated_at: Some(date_at_or_placeholder(m.updated_at)),
                status: m.status.clone(),
                provider_id: m.provider_id,
                cancelled_at: Some(date_at_or_placeholder(m.cancelled_at)),
                shipped_at: Some(date_at_or_placeholder(m.shipped_at)),
                estimated_date: Some(estimated_date_placeholder(
                    m.estimated_date.map(|dt| dt.date()),
                )),
            }
        })
        .collect();

    let path_meta = <&str>::clone(&path);
    let vector_for_schema = &records;
    let schema = vector_for_schema.as_slice().schema().unwrap();
    println!("{:?} schema", &schema);
    // let props = Arc::new(WriterProperties::builder().build());

    let file = std::fs::File::create(path).unwrap();
    let mut pfile = SerializedFileWriter::new(file, schema, props()).unwrap();

    {
        let mut row_group = pfile.next_row_group().unwrap();
        (&records[..])
            .write_to_row_group(&mut row_group)
            .expect("can't write_to_row_group");
        pfile.close_row_group(row_group).unwrap();
    }

    // let rows_number = *pfile.total_num_rows() as i64;
    pfile.close().unwrap();
    let reader = SerializedFileReader::try_from(path_meta).unwrap();
    let parquet_metadata = reader.metadata();
    let file_metadata = parquet_metadata.file_metadata();
    let rows_number = file_metadata.num_rows();
    (path.into(), rows_number)
}

pub struct MilestoneTask {}

impl DraculaTask for MilestoneTask {
    fn run(&self, postgres_uri: &str) -> (String, i64) {
        milestones(postgres_uri)
    }
}
