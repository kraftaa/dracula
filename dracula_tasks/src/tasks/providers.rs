use super::prelude::*;
// use uuid::Uuid;

use dracula_schemas::tables::providers::dsl as providers_dsl;

#[derive(ParquetRecordWriter)]

struct ProviderRecord {
    id: i32,
    name: Option<String>,
    slug: Option<String>,
    uuid: Option<String>,
    created_at: Option<NaiveDateTime>,
    certifications: String,
    percent_completeness: i32,
    score: f64,
    tier: Option<i32>,
    contact_emails: String,
    sales_email: Option<String>,
    phone_number: Option<String>,
    ads_advertiser_id: Option<i32>,
    website: Option<String>,
    number_of_employees: Option<String>,
    type_of_company: Option<String>,
}

pub fn providers(pg_uri: &str) -> (String, i64) {
    let conn = PgConnection::establish(pg_uri).unwrap();

    let providers_load = Instant::now();
    let providers = providers_dsl::providers.load::<Provider>(&conn).unwrap();
    trace!("load providers took: {:?}", providers_load.elapsed());

    let percentage_rates: HashMap<&str, i32> = [
        ("Create Account", 20),
        ("Update Company Profile", 20),
        ("Add Addresses", 20),
        ("Add Certifications", 20),
        ("Add Accounting Details", 20),
    ]
    .iter()
    .cloned()
    .collect();

    let path = "/tmp/providers.parquet";

    let records: Vec<ProviderRecord> = providers
        .iter()
        .map(|p| {
            let certifications = p.certifications.clone().join(", ");

            let setup = p.setup.clone().join(", ");

            let mut percent_completeness: i32 = 0;

            for i in setup.split(',') {
                percent_completeness += percentage_rates.get(i.trim_start()).unwrap_or(&0);
            }

            let contact_emails = p.contact_emails.clone().unwrap().join(", ");

            ProviderRecord {
                id: p.id,
                name: p.name.clone(),
                slug: p.slug.clone(),
                uuid: Some(p.uuid.unwrap().to_string()),
                created_at: p.created_at,
                certifications,
                percent_completeness,
                score: p.score,
                tier: p.tier,
                contact_emails,
                sales_email: p.sales_email.clone(),
                phone_number: p.phone_number.clone(),
                ads_advertiser_id: p.ads_advertiser_id,
                website: p.website.clone(),
                number_of_employees: p.number_of_employees.clone(),
                type_of_company: p.type_of_company.clone(),
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

pub struct ProviderTask {}

impl DraculaTask for ProviderTask {
    fn run(&self, postgres_uri: &str) -> (String, i64) {
        providers(postgres_uri)
    }
}
