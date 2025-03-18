use chrono::NaiveDateTime;
use dracula_parquet::dracula_parquet::prelude::*;
use dracula_parquet::prelude::{ParquetRecordWriter, SerializedFileReader, SerializedFileWriter};
use dracula_parquet::props;
use dracula_tasks::tasks::prelude::*;
pub use futures_util::stream::StreamExt;
pub use sqlx::postgres::PgPool;

#[derive(ParquetRecordWriter, sqlx::FromRow, Default, Debug)]
struct ClickStreamRecord {
    id: String,
    metaschema: String,
    burnin: Option<bool>,
    device: Option<String>,
    passid: Option<i32>,
    siteid: Option<i32>,
    zoneid: Option<i32>,
    brandid: Option<i32>,
    adtypeid: Option<i32>,
    location: Option<String>,
    ratetype: Option<i32>,
    channelid: Option<i32>,
    created_on: Option<NaiveDateTime>,
    networkid: Option<i32>,
    isnotrack: Option<bool>,
    campaignid: Option<i32>,
    clickcount: Option<i32>,
    creativeid: Option<i32>,
    datacenter: Option<bool>,
    decisionid: Option<String>,
    priorityid: Option<i32>,
    timezoneid: String,
    decisionidx: Option<i32>,
    servedbyasg: Option<String>,
    servedbypid: Option<i32>,
    impressionid: Option<String>,
    placementname: Option<String>,
    creativepassid: Option<i32>,
    eventcreatedon: Option<NaiveDateTime>,
    firstchannelid: Option<i32>,
    impressioncreatedon: Option<NaiveDateTime>,
    keywords: Option<String>,
    price: Option<String>,
    url: Option<String>,
    user_key: Option<String>,
    user_is_new: Option<bool>,
    user_type: Option<i32>,
    file_name: String,
    last_modified_date: NaiveDateTime,
    year: i32,
}

pub async fn clicks_stream_task(pg_uri: &str) -> anyhow::Result<(String, i64)> {
    let pool = PgPool::connect(pg_uri).await?;

    let p1 = vec![ClickStreamRecord {
        ..Default::default()
    }];
    let vector_for_schema = &p1;
    let schema = vector_for_schema.as_slice().schema().unwrap();
    let schema_2 = vector_for_schema.as_slice().schema().unwrap();
    let schema_vec = schema_2.get_fields();

    let mut fields: Vec<&str> = vec![];
    for i in schema_vec {
        if i.name() == "id" {
            fields.push("data ->> 'Id' as id")
        } else if i.name() == "metaschema" {
            fields.push("data ->> 'Meta:schema' as metaschema")
        } else if i.name() == "burnin" {
            fields.push("(data ->> 'BurnIn')::Bool as burnin")
        } else if i.name() == "device" {
            fields.push("data ->> 'Device' as device")
        } else if i.name() == "passid" {
            fields.push("(data ->> 'PassId')::INT4 as passid")
        } else if i.name() == "siteid" {
            fields.push("(data ->> 'SiteId')::INT4 as siteid")
        } else if i.name() == "zoneid" {
            fields.push("(data ->> 'ZoneId')::INT4 as zoneid")
        } else if i.name() == "brandid" {
            fields.push("(data ->> 'BrandId')::INT4 as brandid")
        } else if i.name() == "adtypeid" {
            fields.push("(data ->> 'AdTypeId')::INT4 as adtypeid")
        } else if i.name() == "location" {
            fields.push("data ->> 'Location' as location")
        } else if i.name() == "ratetype" {
            fields.push("(data ->> 'RateType')::INT4 as ratetype")
        } else if i.name() == "channelid" {
            fields.push("(data ->> 'ChannelId')::INT4 as channelid")
        } else if i.name() == "created_on" {
            fields.push(r"to_timestamp(CAST(substring((data ->> 'CreatedOn') from 'Date\(([0-9]+)') AS BIGINT)/1000.00)  AT TIME ZONE 'UTC'  as created_on")
        } else if i.name() == "networkid" {
            fields.push("(data ->> 'NetworkId')::INT4 as networkid")
        } else if i.name() == "isnotrack" {
            fields.push("(data ->> 'IsNoTrack')::Bool as isnotrack")
        } else if i.name() == "campaignid" {
            fields.push("(data ->> 'CampaignId')::INT4 as campaignid")
        } else if i.name() == "clickcount" {
            fields.push("(data ->> 'ClickCount')::INT4 as clickcount")
        } else if i.name() == "creativeid" {
            fields.push("(data ->> 'CreativeId')::INT4 as creativeid")
        } else if i.name() == "datacenter" {
            fields.push("(data ->> 'DataCenter')::Bool as datacenter")
        } else if i.name() == "decisionid" {
            fields.push("data ->> 'DecisionId' as decisionid")
        } else if i.name() == "priorityid" {
            fields.push("(data ->> 'PriorityId')::INT4 as priorityid")
        } else if i.name() == "timezoneid" {
            fields.push("data ->> 'TimeZoneId' as timezoneid")
        } else if i.name() == "decisionidx" {
            fields.push("(data ->> 'DecisionIdx')::INT4 as decisionidx")
        } else if i.name() == "servedbyasg" {
            fields.push("data ->> 'ServedByAsg' as servedbyasg")
        } else if i.name() == "servedbypid" {
            fields.push("(data ->> 'ServedByPid')::INT4 as servedbypid")
        } else if i.name() == "impressionid" {
            fields.push("data ->> 'ImpressionId' as impressionid")
        } else if i.name() == "placementname" {
            fields.push("data ->> 'PlacementName' as placementname")
        } else if i.name() == "creativepassid" {
            fields.push("(data ->> 'CreativePassId')::INT4 as creativepassid")
        } else if i.name() == "eventcreatedon" {
            fields.push(r"to_timestamp(CAST(data ->> 'EventCreatedOn' AS BIGINT)/1000.00)  AT TIME ZONE 'UTC'  as eventcreatedon")
        } else if i.name() == "firstchannelid" {
            fields.push("(data ->> 'FirstChannelId')::INT4 as firstchannelid")
        } else if i.name() == "impressioncreatedon" {
            fields.push(r"to_timestamp(CAST(data ->> 'ImpressionCreatedOn' AS BIGINT)/1000.00)  AT TIME ZONE 'UTC'  as impressioncreatedon")
        } else if i.name() == "keywords" {
            fields.push("data ->> 'Keywords' as keywords")
        } else if i.name() == "price" {
            fields.push("data ->> 'Price' as price")
        } else if i.name() == "url" {
            fields.push("data ->> 'Url' as url")
        } else if i.name() == "user_key" {
            fields.push("(( data ->> 'User')::json ->> 'Key')::Varchar as user_key")
        } else if i.name() == "user_is_new" {
            fields.push("(( data ->> 'User')::json ->> 'IsNew')::Bool as user_is_new")
        } else if i.name() == "user_type" {
            fields.push("(( data ->> 'User')::json ->> 'Type')::INT4 as user_type")
        } else if i.name() == "file_name" {
            fields.push("file_name")
        } else if i.name() == "last_modified_date" {
            fields.push("last_modified_date")
        } else if i.name() == "year" {
            fields.push("year")
        } else {
            fields.push(i.name())
        }
    }

    println!("{:?} fields!", fields);

    println!("{:?} schema", &schema);

    let path = "/tmp/clicks.parquet";

    let path_meta = <&str>::clone(&path);

    let file = std::fs::File::create(path).unwrap();
    let mut pfile = SerializedFileWriter::new(file, schema, props()).unwrap();
    let table: &str = "clicks";

    let mut query = "SELECT ".to_owned();
    let fields: &str = &fields.join(", ");
    query.push_str(fields);
    query.push_str(" FROM ");
    query.push_str(table);

    let q = sqlx::query_as::<sqlx::Postgres, ClickStreamRecord>(&query);

    let cl_stream = q.fetch(&pool);
    println!("{} query", query);

    let mut chunk_stream = cl_stream.map(|fs| fs.unwrap()).chunks(5000);
    // `futures_util::stream::Map<Pin<Box<dyn futures_util::Stream<Item = Result<FeatureSetting, sqlx::Error>> + std::marker::Send>>
    while let Some(chunks) = chunk_stream.next().await {
        let mut row_group = pfile.next_row_group().unwrap();
        (&chunks[..])
            .write_to_row_group(&mut row_group)
            .expect("can't 'write_to_row_group' ...");
        pfile.close_row_group(row_group).unwrap();
    }

    pfile.close().unwrap();
    let reader = SerializedFileReader::try_from(path_meta).unwrap();
    let parquet_metadata = reader.metadata();
    let file_metadata = parquet_metadata.file_metadata();
    let rows_number = file_metadata.num_rows();
    Ok((path.into(), rows_number))
}

use async_trait::async_trait;
#[derive(Debug)]
pub struct ClickStreamTask {}
#[async_trait]
impl DraculaStreamingTask for ClickStreamTask {
    async fn run(&self, postgres_uri: &str) -> (String, i64) {
        clicks_stream_task(postgres_uri).await.unwrap()
    }
}
