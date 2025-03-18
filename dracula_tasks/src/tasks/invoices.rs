use super::prelude::*;
use std::str;
// use uuid::Uuid;

use dracula_schemas::tables::{
    currencies::dsl as currencies_dsl, invoices::dsl as invoices_dsl,
    shippings::dsl as shippings_dsl, taxes::dsl as taxes_dsl,
};
#[derive(ParquetRecordWriter)]
struct InvoiceRecord {
    id: i32,
    invoice_number: Option<String>,
    issued_at: Option<NaiveDateTime>,
    purchase_order_id: Option<i32>,
    created_at: Option<NaiveDateTime>,
    created_on: Option<NaiveDate>,
    subtotal_price: f64,
    total_price: f64,
    discount_amount: f64,
    tax: f64,
    currency_name: String,
    conversion_rate: f64,
    document_type: String,
    total_price_usd: f64,
    provider_id: Option<i32>,
    order_id: Option<i32>,
    shipping_cost: f64,
    cancelled: String,
    uuid: Option<String>,
    approved_for_payment: Option<bool>,
    discount: Option<String>,
}

pub fn invoices(pg_uri: &str) -> (String, i64) {
    let conn = PgConnection::establish(pg_uri).unwrap();

    let invoices_load = Instant::now();
    let invoices = invoices_dsl::invoices.load::<Invoice>(&conn).unwrap();
    trace!("load invoices took: {:?}", invoices_load.elapsed());

    let currency_ids: Vec<_> = invoices.iter().map(|x| x.id).collect();
    let currencies_load = Instant::now();
    let currencies: Vec<Currency> = currencies_dsl::currencies
        .filter(currencies_dsl::exchangable_type.eq("Pg::Invoice"))
        .filter(currencies_dsl::exchangable_id.eq(any(&currency_ids[..])))
        .load::<Currency>(&conn)
        .unwrap();
    trace!(
        "load currencies ({}) took: {:?}",
        currencies.len(),
        currencies_load.elapsed()
    );

    let _currencies_by_invoice_id: HashMap<i32, &Currency> = currencies
        .iter()
        .map(|x| (x.exchangable_id.unwrap(), x))
        .collect();

    let taxes_load = Instant::now();
    let taxes: Vec<Tax> = taxes_dsl::taxes
        .filter(taxes_dsl::taxable_type.eq("Pg::Invoice"))
        .filter(taxes_dsl::type_.eq("Pg::RetailTax"))
        .filter(taxes_dsl::taxable_id.eq(any(&currency_ids[..])))
        .load::<Tax>(&conn)
        .unwrap();
    trace!(
        "load taxes_time ({}) took: {:?}",
        taxes.len(),
        taxes_load.elapsed()
    );

    let _taxes_by_invoice_id: HashMap<i32, &Tax> =
        taxes.iter().map(|x| (x.taxable_id.unwrap(), x)).collect();

    let shippings_load = Instant::now();
    let shippings: Vec<Shipping> = shippings_dsl::shippings
        .filter(shippings_dsl::shipable_type.eq("Pg::Invoice"))
        .filter(shippings_dsl::shipable_id.eq(any(&currency_ids[..])))
        .load::<Shipping>(&conn)
        .unwrap();
    trace!(
        "load shippings ({}) took: {:?}",
        shippings.len(),
        shippings_load.elapsed()
    );

    let path = "/tmp/invoices.parquet";
    // let mut pfile = dracula_parquet::parquet_writer::<InvoiceRecord>(path).unwrap();

    let records: Vec<InvoiceRecord> = invoices
        .iter()
        .map(|i| {
            let conversion_rate = currencies
                .iter()
                .find(|x| x.exchangable_id.unwrap() == i.id)
                .expect("find exchangeable_id")
                .conversion_rate
                .clone()
                .map(|x| x.to_f64().expect("bigdecimal to f64"))
                .unwrap_or(1.0);
            let currency_name = currencies
                .iter()
                .find(|x| x.exchangable_id.unwrap() == i.id)
                .expect("find exchangeable_id")
                .currency
                .clone()
                .unwrap_or_else(|| "USD".to_string());
            let mut shipping_cost = shippings
                .iter()
                .find(|y| y.shipable_id.unwrap() == i.id)
                .expect("find shipable_id")
                .cost
                .to_f64()
                .expect("bigdecimal to f64");
            let subtotal_price = if &i.document_type[..] == "standard" {
                i.subtotal_price.to_f64().expect("big decimal price")
            } else {
                -i.subtotal_price.to_f64().expect("big decimal price")
            };
            let total_price = if &i.document_type[..] == "standard" {
                i.total_price.to_f64().expect("big decimal price")
            } else {
                -i.total_price.to_f64().expect("big decimal price")
            };

            if total_price < 0.0 {
                shipping_cost = -shipping_cost
            }
            let discount_amount = i.discount_amount.to_f64().expect("big decimal price");

            let tax = total_price - subtotal_price - shipping_cost + discount_amount;
            // changing logic for converted price
            let mut total_price_usd = total_price / conversion_rate;
            if total_price_usd < 0.0 {
                total_price_usd = -total_price_usd
            }
            let cancelled = i.cancelled.to_string();
            let document_type = i.document_type.clone();

            InvoiceRecord {
                id: i.id,
                invoice_number: i.invoice_number.clone(),
                issued_at: i.issued_at,
                purchase_order_id: i.purchase_order_id,
                created_at: i.created_at,
                created_on: i.created_at.map(|ca| ca.date()),
                subtotal_price,
                total_price,
                discount_amount,
                tax,
                currency_name,
                conversion_rate,
                document_type,
                total_price_usd,
                provider_id: i.provider_id,
                order_id: i.order_id,
                shipping_cost,
                cancelled,
                uuid: Some(i.uuid.unwrap().to_string()),
                approved_for_payment: i.approved_for_payment,
                discount: i.discount.clone(),
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

pub struct InvoiceTask {}

impl DraculaTask for InvoiceTask {
    fn run(&self, postgres_uri: &str) -> (String, i64) {
        invoices(postgres_uri)
    }
}
