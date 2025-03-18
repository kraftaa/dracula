use super::prelude::*;
use std::collections::HashMap;
// use uuid::Uuid;

use dracula_schemas::tables::{
    currencies::dsl as currencies_dsl, proposals::dsl as pr_dsl, providers::dsl as providers_dsl,
    purchase_orders::dsl as purchase_orders_dsl, taxes::dsl as taxes_dsl,
    turn_around_times::dsl as turn_around_times_dsl,
};

#[derive(ParquetRecordWriter)]
struct ProposalsRecordPage {
    id: i32,
    active: String,
    proposal_type: Option<String>,
    exclude: String,
    justifications: String,
    user_full_name: Option<String>,
    provider_id: Option<i32>,
    provider_name: Option<String>,
    created_at: Option<NaiveDateTime>,
    created_on: Option<NaiveDate>,
    updated_at: Option<NaiveDateTime>,
    subtotal_price: f64,
    total_price: f64,
    tax: Option<f64>,
    total_price_no_tax: f64,
    currency_name: String,
    conversion_rate: f64,
    total_price_usd: f64,
    total_price_with_tax_usd: f64,
    min_turnaround_time: Option<i64>,
    max_turnaround_time: Option<i64>,
    display_units: Option<String>,
    tax_category: Option<String>,
    order_id: Option<i32>,
    type_: Option<String>,
    obsoleted: String,
    state: Option<String>,
    ware_id: Option<i32>,
    user_id: Option<i32>,
    uuid: Option<String>,
    description: String,
    commission_rate: Option<f64>,
    status: Option<String>,
}

pub fn proposals(pg_uri: &str) -> (String, i64) {
    let conn = PgConnection::establish(pg_uri).unwrap();
    let proposals_load = Instant::now();
    let proposals = pr_dsl::proposals.load::<Proposal>(&conn).unwrap();

    let path = "/tmp/proposals.parquet";
    let path_meta = <&str>::clone(&path);

    let mut count = 0;

    trace!("load proposals took: {:?}", proposals_load.elapsed());

    let proposal_ids: Vec<i32> = proposals.iter().map(|x| x.id).collect();
    trace!("{:?}", proposal_ids.len());
    let provider_ids: Vec<i32> = proposals.iter().filter_map(|x| x.provider_id).collect();
    trace!("{:?}", provider_ids.len());

    let currencies_load = Instant::now();
    let currencies: Vec<Currency> = currencies_dsl::currencies
        .filter(currencies_dsl::exchangable_type.eq("Pg::ProposalBase"))
        .filter(currencies_dsl::exchangable_id.eq(any(&proposal_ids[..])))
        .load::<Currency>(&conn)
        .unwrap();
    trace!(
        "load currencies ({}) took: {:?}",
        currencies.len(),
        currencies_load.elapsed()
    );

    let currencies_by_proposal_id: HashMap<i32, &Currency> = currencies
        .iter()
        .map(|x| (x.exchangable_id.unwrap(), x))
        .collect();

    let turn_around_times_load = Instant::now();
    let turn_around_times: Vec<TurnAroundTime> = turn_around_times_dsl::turn_around_times
        .filter(turn_around_times_dsl::turnaroundable_type.eq("Pg::ProposalBase"))
        .filter(turn_around_times_dsl::turnaroundable_id.eq(any(&proposal_ids[..])))
        .load::<TurnAroundTime>(&conn)
        .unwrap();
    trace!(
        "load turn_around_time ({}) took: {:?}",
        turn_around_times.len(),
        turn_around_times_load.elapsed()
    );

    let turn_around_times_by_proposal_id: HashMap<i32, &TurnAroundTime> = turn_around_times
        .iter()
        .map(|x| (x.turnaroundable_id.unwrap(), x))
        .collect();

    let _po_load = Instant::now();
    let pos: Vec<PurchaseOrder> = purchase_orders_dsl::purchase_orders
        .filter(purchase_orders_dsl::po_number.is_not_null())
        .filter(purchase_orders_dsl::proposal_id.is_not_null())
        .filter(purchase_orders_dsl::proposal_id.eq(any(&proposal_ids[..])))
        .load::<PurchaseOrder>(&conn)
        .unwrap();
    let _po_by_proposal_id: HashMap<i32, &PurchaseOrder> =
        pos.iter().map(|x| (x.proposal_id.unwrap(), x)).collect();

    let taxes_load = Instant::now();
    let taxes: Vec<Tax> = taxes_dsl::taxes
        .filter(taxes_dsl::taxable_type.eq("Pg::ProposalBase"))
        .filter(taxes_dsl::type_.eq("Pg::RetailTax"))
        .filter(taxes_dsl::taxable_id.eq(any(&proposal_ids[..])))
        .load::<Tax>(&conn)
        .unwrap();
    trace!(
        "load taxes_time ({}) took: {:?}",
        taxes.len(),
        taxes_load.elapsed()
    );

    let taxes_by_proposal_id: HashMap<i32, &Tax> =
        taxes.iter().map(|x| (x.taxable_id.unwrap(), x)).collect();

    let providers_load = Instant::now();
    let providers: Vec<Provider> = providers_dsl::providers
        .filter(providers_dsl::id.eq(any(&provider_ids[..])))
        .load::<Provider>(&conn)
        .unwrap();
    trace!(
        "load providers ({}) took: {:?}",
        providers.len(),
        providers_load.elapsed()
    );

    let providers_by_purchase_order_id: HashMap<i32, &Provider> =
        providers.iter().map(|x| (x.id, x)).collect();

    // let parquet_records: Vec<ProposalsRecordPage> = res
    let parquet_records: Vec<ProposalsRecordPage> = proposals
        .iter()
        .filter(|pr| pr.active)
        .filter(|pr| pr.order_id.is_some())
        .map(|p| {
            let currency = currencies_by_proposal_id.get(&p.id);
            let conversion_rate = currency
                .map(|x| {
                    x.conversion_rate
                        .clone()
                        .map(|cr| cr.to_f64().expect("bigdecimal to f64"))
                        //                        .unwrap()
                        .expect("Unwrapping currency in Proposals")
                })
                .unwrap_or(1.0);

            let currency_name = currency
                .map(|x| {
                    x.currency
                        .clone()
                        .expect("Unwrapping currency name in Proposals")
                })
                .unwrap_or_else(|| "USD".to_string());

            let taxes = taxes_by_proposal_id.get(&p.id);
            let tax = taxes.map(|x| x.amount.to_f64().expect("bigdecimal to f64"));

            let tax_date: NaiveDateTime = NaiveDate::from_ymd_opt(2018, 6, 1)
                .unwrap()
                .and_hms_milli_opt(0, 0, 0, 0)
                .unwrap();

            let total_price = p.total_price.to_f64().expect("big decimal price");
            let subtotal_price = p.subtotal_price.to_f64().expect("big decimal price");

            let total_price_no_tax = if p.created_at > Some(tax_date) {
                total_price - tax.unwrap_or(0.0)
            } else {
                total_price
            };

            let total_price_usd = total_price_no_tax / conversion_rate;
            let total_price_with_tax_usd = total_price / conversion_rate;
            let provider = providers_by_purchase_order_id.get(&p.provider_id.unwrap_or(0));

            let provider_name = if let Some(p) = provider {
                p.name.clone()
            } else {
                None
            };

            let active = p.active.to_string();
            let exclude = p.exclude.to_string();

            let turn_around_time = turn_around_times_by_proposal_id.get(&p.id);
            let min_turnaround_time = turn_around_time
                .map(|x| x.min.map(|x| x.to_i64().expect("bigint to i64")))
                .unwrap_or(Some(0));

            let max_turnaround_time = turn_around_time
                .map(|x| x.max.map(|x| x.to_i64().expect("bigint to i64")))
                .unwrap_or(Some(0));

            let display_units = turn_around_time
                .map(|x| x.display_units.clone())
                .unwrap_or_else(|| Some("None".to_string()));

            let justifications = p.justifications.clone().unwrap().join(", ");

            let obsoleted = p.obsoleted.to_string();
            let state = p.state.clone();
            let commission_rate = p.commission_rate.as_ref().map(|l| l.to_f64().unwrap());

            ProposalsRecordPage {
                id: p.id,
                active,
                proposal_type: p.proposal_type.clone(),
                exclude,
                justifications,
                user_full_name: p.user_full_name.clone(),
                provider_id: p.provider_id,
                provider_name,
                created_at: p.created_at,
                created_on: p.created_at.map(|ca| ca.date()),
                updated_at: p.updated_at,
                total_price,
                subtotal_price,
                tax,
                total_price_no_tax,
                currency_name,
                conversion_rate,
                total_price_usd,
                total_price_with_tax_usd,
                min_turnaround_time,
                max_turnaround_time,
                display_units,
                tax_category: p.tax_category.clone(),
                order_id: p.order_id,
                type_: p.type_.clone(),
                obsoleted,
                state,
                ware_id: p.ware_id,
                user_id: p.user_id,
                uuid: Some(p.uuid.unwrap().to_string()),
                description: p.description.clone(),
                commission_rate,
                status: p.status.clone(),
            }
        })
        .collect();

    let vector_for_schema = &parquet_records;
    let schema = vector_for_schema.as_slice().schema().unwrap();
    println!("{:?} schema", &schema);
    // let props = Arc::new(WriterProperties::builder().build());

    let file = std::fs::File::create(path).unwrap();
    // pfile = SerializedFileWriter::new(file, schema, props()).unwrap();
    let mut pfile = SerializedFileWriter::new(file, schema, props()).unwrap();

    {
        let mut row_group = pfile.next_row_group().unwrap();
        (&parquet_records[..])
            .write_to_row_group(&mut row_group)
            .expect("can't 'write_to_row_group' ...");
        pfile.close_row_group(row_group).unwrap();
        count += 1;
        println!("{} count", count);
    }
    // }

    // let rows_number = *pfile.total_num_rows() as i64;
    pfile.close().unwrap();
    let reader = SerializedFileReader::try_from(path_meta).unwrap();
    let parquet_metadata = reader.metadata();
    let file_metadata = parquet_metadata.file_metadata();
    let rows_number = file_metadata.num_rows();
    (path.into(), rows_number)
}

pub struct ProposalPageTask {}

impl DraculaTask for ProposalPageTask {
    fn run(&self, postgres_uri: &str) -> (String, i64) {
        proposals(postgres_uri)
    }
}
