use super::prelude::*;
// use crate::tasks::prelude::parquet::file::properties::WriterProperties;
use dracula_schemas::tables::addresses::dsl as addresses_dsl;

#[derive(ParquetRecordWriter)]
struct TinyAddress {
    id: i32,
    type_: Option<String>,
    organization_name: Option<String>,
    site_name: Option<String>,
    attention: Option<String>,
    person_name: Option<String>,
    street: Option<String>,
    street2: Option<String>,
    city: Option<String>,
    state: Option<String>,
    zipcode: Option<String>,
    country: Option<String>,
    created_at: Option<NaiveDateTime>,
    addressable_id: Option<i32>,
    addressable_type: Option<String>,
    legal_entity_id: Option<i32>,
}

impl From<&Address> for TinyAddress {
    fn from(incoming: &Address) -> Self {
        TinyAddress {
            id: incoming.id,
            type_: incoming.type_.clone(),
            organization_name: incoming.organization_name.clone(),
            site_name: incoming.site_name.clone(),
            attention: incoming.attention.clone(),
            person_name: incoming.person_name.clone(),
            street: incoming.street.clone(),
            street2: incoming.street2.clone(),
            city: incoming.city.clone(),
            state: incoming.state.clone(),
            zipcode: incoming.zipcode.clone(),
            country: incoming.country.clone(),
            created_at: incoming.created_at,
            addressable_id: incoming.addressable_id,
            addressable_type: incoming.addressable_type.clone(),
            legal_entity_id: incoming.legal_entity_id,
        }
    }
}
//#[named]
pub fn addresses(pg_uri: &str) -> (String, i64) {
    let conn = PgConnection::establish(pg_uri).unwrap();

    let addressesshort_load = Instant::now();
    let addresses: Vec<TinyAddress> = addresses_dsl::addresses
        .load::<Address>(&conn)
        .unwrap()
        .iter()
        .map(|addy| addy.into())
        .collect();
    trace!("load addresses took: {:?}", addressesshort_load.elapsed());

    let path = "/tmp/addresses.parquet";
    let path_meta = <&str>::clone(&path);

    let vector_for_schema = &addresses;
    let schema = vector_for_schema.as_slice().schema().unwrap();
    println!("{:?} schema", &schema);
    // let props = Arc::new(WriterProperties::builder().build());

    let file = std::fs::File::create(path).unwrap();
    let mut pfile = SerializedFileWriter::new(file, schema, props()).unwrap();
    {
        let mut row_group = pfile.next_row_group().unwrap();
        (&addresses[..])
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
    (path.into(), rows_number)
}

pub struct AddressTask {}

impl DraculaTask for AddressTask {
    fn run(&self, postgres_uri: &str) -> (String, i64) {
        addresses(postgres_uri)
    }
}
