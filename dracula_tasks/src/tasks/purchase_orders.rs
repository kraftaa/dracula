use super::prelude::*;
use dracula_schemas::tables::{
    action_items::dsl as action_items_dsl, currencies::dsl as currencies_dsl,
    providers::dsl as providers_dsl, purchase_orders::dsl as po_dsl,
    orders::dsl as orders_dsl,
    shippings::dsl as shippings_dsl, taxes::dsl as taxes_dsl,
};
use rayon::prelude::*;
use std::str;
// use uuid::Uuid;

#[derive(ParquetRecordWriter)]
struct PurchaseOrderTaskRecord {
    id: i32,
    ad_po_number: Option<String>,
    po_number: Option<String>,
    po_created_at: Option<NaiveDateTime>,
    po_submission_date: Option<NaiveDate>,
    status: Option<String>,
    created_at: Option<NaiveDateTime>,
    proposal_id: Option<i32>,
    total_price: f64,
    subtotal_price: f64,
    provider_id: Option<i32>,
    provider_name: Option<String>,
    supplier_id: Option<String>,
    justifications: String,
    currency: Option<String>,
    conversion_rate: f64,
    total_price_usd: f64,
    subtotal_price_usd: f64,
    tax_amount: f64,
    tax_percent: f64,
    tax_amount_usd: f64,
    shipping_cost: f64,
    shipping_cost_usd: f64,
    commission_rate: f64,
    order_id: Option<i32>,
    provider_shipping_address_id: Option<i32>,
    customer_shipping_address_id: Option<i32>,
    cpo_id: Option<i32>,
    uuid: Option<String>,
    rejected: Option<String>,
    obsoleted: Option<String>,
    type_: Option<String>,
    supplier_updated: Option<bool>,
    updated_at: Option<NaiveDateTime>,
    user_id: Option<i32>,
    payment_terms: Option<i32>,
    request_id: Option<i32>,
}

pub fn purchase_orders(pg_uri: &str) -> (String, i64) {
    let conn = PgConnection::establish(pg_uri).unwrap();

    let purchase_orders_load = Instant::now();

    let purchase_orders = po_dsl::purchase_orders
        .filter(po_dsl::type_.eq_any(["Pg::CustomerPurchaseOrder", "Pg::PurchaseOrder"]))
        .load::<PurchaseOrder>(&conn)
        .unwrap();
    trace!(
        "load purchase orders took: {:?}",
        purchase_orders_load.elapsed()
    );

    let cpo_ids: Vec<_> = purchase_orders.iter().map(|x| x.id).collect();
    let ppo_base_load = Instant::now();
    let ppo_base = po_dsl::purchase_orders
        .filter(po_dsl::type_.eq("Pg::ProviderPurchaseOrder"))
        .filter(po_dsl::cpo_id.eq(any(&cpo_ids[..])))
        .load::<PurchaseOrder>(&conn)
        .unwrap();
    println!(
        "load provider_purchase_order ({}) took: {:?}",
        ppo_base.len(),
        ppo_base_load.elapsed()
    );

    let ad_po_number_by_purchase_order_id: HashMap<i32, &PurchaseOrder> = ppo_base
        .iter()
        .map(|x| (x.cpo_id.unwrap(), x))
        .collect();
    println!(
        "Length of HashMap ad_po_number_by_purchase_order_id: {}\n",
        ad_po_number_by_purchase_order_id.len()
    );
    let currency_ids: Vec<_> = purchase_orders.iter().map(|x| x.id).collect();
    let currencies_load = Instant::now();
    let currencies: Vec<Currency> = currencies_dsl::currencies
        .filter(
            currencies_dsl::exchangable_type
                .eq_any(["Pg::PurchaseOrderBase", "Pg::PurchaseOrderBaseInit"]),
        )
        .filter(currencies_dsl::exchangable_id.eq(any(&currency_ids[..])))
        .load::<Currency>(&conn)
        .unwrap();
    trace!(
        "load currencies ({}) took: {:?}",
        currencies.len(),
        currencies_load.elapsed()
    );

    let currencies_by_purchase_order_id: HashMap<i32, &Currency> = currencies
        .iter()
        .map(|x| (x.exchangable_id.unwrap(), x))
        .collect();

    let provider_ids: Vec<_> = purchase_orders
        .iter()
        .filter_map(|x| x.provider_id)
        .collect();
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

    let order_ids: Vec<_> = purchase_orders
        .iter()
        .filter_map(|x| x.order_id)
        .collect();

    let orders_load = Instant::now();
    let orders: Vec<Order> = orders_dsl::orders
        .filter(orders_dsl::id.eq(any(&order_ids[..])))
        .load::<Order>(&conn)
        .unwrap();
    trace!(
        "load orders ({}) took: {:?}",
        orders.len(),
        orders_load.elapsed()
    );

    let action_items_load = Instant::now();
    let action_items: Vec<ActionItem> = action_items_dsl::action_items
        .filter(action_items_dsl::action_performed.eq("new_request"))
        .filter(action_items_dsl::order_id.eq(any(&order_ids[..])))
        .load::<ActionItem>(&conn)
        .unwrap();
    trace!(
        "load action items ({}) took: {:?}",
        action_items.len(),
        action_items_load.elapsed()
    );

    let _action_items_purchase_order_id: HashMap<&i32, &ActionItem> = action_items
        .iter()
        .map(|x| (x.order_id.as_ref().unwrap(), x))
        .collect();

    let shippings_load = Instant::now();
    let shippings: Vec<Shipping> = shippings_dsl::shippings
        // .filter(shippings_dsl::shipable_type.eq("Pg::PurchaseOrderBase"))
        .filter(
            shippings_dsl::shipable_type
                .eq_any(["Pg::PurchaseOrderBase", "Pg::PurchaseOrderBaseInit"]),
        )
        .filter(shippings_dsl::shipable_id.eq(any(&currency_ids[..])))
        .load::<Shipping>(&conn)
        .unwrap();
    trace!(
        "load shippings ({}) took: {:?}",
        shippings.len(),
        shippings_load.elapsed()
    );

    let taxes_load = Instant::now();
    let taxes: Vec<Tax> = taxes_dsl::taxes
        .filter(
            taxes_dsl::taxable_type.eq_any(["Pg::PurchaseOrderBase", "Pg::PurchaseOrderBaseInit"]),
        )
        .filter(taxes_dsl::type_.eq("Pg::RetailTax"))
        .filter(taxes_dsl::taxable_id.eq(any(&currency_ids[..])))
        .load::<Tax>(&conn)
        .unwrap();
    trace!(
        "load taxes_time ({}) took: {:?}",
        taxes.len(),
        taxes_load.elapsed()
    );

    let taxes_by_po_id: HashMap<i32, &Tax> =
        taxes.iter().map(|x| (x.taxable_id.unwrap(), x)).collect();

    let proposal_ids: Vec<_> = purchase_orders
        .iter()
        .map(|x| Some(x.proposal_id.unwrap_or(0)))
        .collect();

    let taxes_proposal_load = Instant::now();
    let taxes_proposal: Vec<Tax> = taxes_dsl::taxes
        .filter(taxes_dsl::taxable_type.eq("Pg::ProposalBase"))
        .filter(taxes_dsl::type_.eq("Pg::RetailTax"))
        .filter(taxes_dsl::taxable_id.eq(any(&proposal_ids[..])))
        .load::<Tax>(&conn)
        .unwrap();
    trace!(
        "load taxes_proposal_time ({}) took: {:?}",
        taxes_proposal.len(),
        taxes_proposal_load.elapsed()
    );

    let taxes_by_proposal_id: HashMap<i32, &Tax> = taxes_proposal
        .iter()
        .map(|x| (x.taxable_id.unwrap(), x))
        .collect();

    let shippings_proposal_load = Instant::now();
    let shippings_proposal: Vec<Shipping> = shippings_dsl::shippings
        .filter(shippings_dsl::shipable_type.eq("Pg::ProposalBase"))
        .filter(shippings_dsl::shipable_id.eq(any(&proposal_ids[..])))
        .load::<Shipping>(&conn)
        .unwrap();
    trace!(
        "load shippings proposals ({}) took: {:?}",
        shippings_proposal.len(),
        shippings_proposal_load.elapsed()
    );

    let shipping_by_proposal_id: HashMap<i32, &Shipping> = shippings_proposal
        .iter()
        .map(|x| (x.shipable_id.unwrap(), x))
        .collect();

    let providers_by_purchase_order_id: HashMap<i32, &Provider> =
        providers.iter().map(|x| (x.id, x)).collect();

    let path = "/tmp/purchase_orders_report.parquet";

    let records_load = Instant::now();

    let records: Vec<PurchaseOrderTaskRecord> = purchase_orders
        .par_iter()
        .filter(|po| {
            po.po_number.is_some() && po.po_created_at.is_some()
        })
        .map(|i| {
            let ad_po_numbers = ad_po_number_by_purchase_order_id.get(&i.id);

            let ppo_ad_po_number = if let Some(ppo) = ad_po_numbers {
                ppo.ad_po_number.clone()
            } else {
                i.ad_po_number.clone()
            };

            let currency = currencies_by_purchase_order_id.get(&i.id);

            let conversion_rate = currency
                .map(|x| {
                    x.conversion_rate
                        .clone()
                        .map(|cr| cr.to_f64().expect("bigdecimal to f64"))
                        .unwrap()
                })
                .unwrap_or(1.0);
            let currency_field = currency.map(|x| x.currency.clone().unwrap());

            let total_price = i.total_price.to_f64().expect("big decimal price");
            let subtotal_price =
                i.subtotal_price.to_f64().expect("big decimal price");

            let shippings_proposal = shipping_by_proposal_id.get(&i.proposal_id.unwrap_or(0));
            let shipping_cost = if shippings
                .iter()
                .find(|y| y.shipable_id.unwrap() == i.id)
                .expect("find shipable_id")
                .cost
                .to_f64()
                .expect("bigdecimal to f64")
                != 0.0
            {
                shippings
                    .iter()
                    .find(|y| y.shipable_id.unwrap() == i.id)
                    .expect("find shipable_id")
                    .cost
                    .to_f64()
                    .expect("bigdecimal to f64")
            } else {
                shippings_proposal
                    .map(|x| x.cost.to_f64().expect("bigdecimal to f64"))
                    .unwrap_or(0.0)
            };

            let shipping_cost_usd = shipping_cost / conversion_rate;

            let total_price_usd = total_price / conversion_rate;
            let subtotal_price_usd = subtotal_price / conversion_rate;
            let provider = providers_by_purchase_order_id.get(&i.provider_id.unwrap_or(0));
            let provider_name = if let Some(p) = provider {
                p.name.clone()
            } else {
                None
            };

            let supplier = action_items
                .iter()
                .find(|z| Some(z.order_id).unwrap() == i.order_id);
            let supplier_id = if let Some(s) = supplier {
                s.provider_id.clone()
            } else {
                None
            };

            let taxes = taxes_by_po_id.get(&i.id);

            let taxes_proposal = taxes_by_proposal_id.get(&i.proposal_id.unwrap_or(0));

            let tax_amount = if taxes.map(|x| &x.amount).is_some()
                && taxes.map(|x| x.amount.to_f64().expect("bigdecimal to f64")) != Some(0.0)
            {
                taxes
                    .map(|x| x.amount.to_f64().expect("bigdecimal to f64"))
                    .unwrap_or(0.0)
            } else {
                taxes_proposal
                    .filter(|r| r.type_.eq(&Some("Pg::RetailTax".to_string())))
                    .map(|x| x.amount.to_f64().expect("bigdecimal to f64"))
                    .unwrap_or(0.0)
            };

            let tax_percent = taxes
                .map(|x| x.percent_rate.to_f64().expect("bigdecimal to f64"))
                .unwrap_or(0.0);

            let tax_amount_usd =
                (tax_amount + shipping_cost * tax_percent / 100.00) / conversion_rate;

            let commission_rate = i
                .commission_rate
                .clone()
                .map(|x| x.to_f64().expect("big decimal rate"))
                .unwrap_or(0.0)
                / 100.0;

            let justifications = i.justifications.clone().join(", ");

            PurchaseOrderTaskRecord {
                id: i.id,
                ad_po_number: ppo_ad_po_number,
                po_number: i.po_number.clone(),
                po_created_at: i.po_created_at,
                po_submission_date: i.po_created_at.map(|co| co.date()),
                status: i.status(),
                created_at: i.created_at,
                proposal_id: i.proposal_id,
                total_price,
                subtotal_price,
                provider_id: i.provider_id,
                provider_name,
                supplier_id,
                justifications,
                currency: currency_field,
                conversion_rate,
                total_price_usd,
                subtotal_price_usd,
                tax_amount,
                tax_percent,
                tax_amount_usd,
                shipping_cost,
                shipping_cost_usd,
                commission_rate,
                order_id: i.order_id,
                provider_shipping_address_id: i.provider_shipping_address_id,
                customer_shipping_address_id: i.customer_shipping_address_id,
                cpo_id: i.cpo_id,
                uuid: Some(i.uuid.unwrap().to_string()),
                rejected: Some(i.rejected.unwrap().to_string()),
                obsoleted: Some(i.obsoleted.unwrap().to_string()),
                type_: i.type_.clone(),
                supplier_updated: i.supplier_updated,
                updated_at: i.updated_at,
                user_id: i.user_id,
                payment_terms: i.payment_terms,
                request_id: i.request_id,
            }
        })
        .collect();

    let path_meta = <&str>::clone(&path);
    let vector_for_schema = &records;
    let schema = vector_for_schema.as_slice().schema().unwrap();
    println!("{:?} schema", &schema);

    let file = std::fs::File::create(path).unwrap();
    let mut pfile = SerializedFileWriter::new(file, schema, props()).unwrap();

    trace!(
        "load records ({}) took: {:?}",
        records.len(),
        records_load.elapsed()
    );

    {
        let mut row_group = pfile.next_row_group().unwrap();
        (&records[..])
            .write_to_row_group(&mut row_group)
            .expect("can't 'write_to_row_group' ...");
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

pub struct PurchaseOrderTask {}

impl DraculaTask for PurchaseOrderTask {
    fn run(&self, postgres_uri: &str) -> (String, i64) {
        purchase_orders(postgres_uri)
    }
}
