use aws_config::BehaviorVersion;
use aws_sdk_athena::client::Client as athena_client;
use aws_sdk_athena::error::SdkError;
use aws_sdk_athena::types::{QueryExecutionContext, ResultConfiguration, ResultSet};
use aws_sdk_glue::operation::get_crawler::GetCrawlerOutput;
use aws_sdk_glue::operation::start_crawler::StartCrawlerError;
use aws_sdk_glue::operation::update_crawler::UpdateCrawlerError;
use aws_sdk_glue::types::{Crawler, CrawlerTargets, S3Target};
use aws_sdk_glue::Error;
use aws_sdk_s3::operation::create_multipart_upload::CreateMultipartUploadOutput;
use aws_sdk_s3::types::{CompletedMultipartUpload, CompletedPart};
use aws_sdk_s3::Client as S3Client;
use aws_smithy_types::byte_stream::{ByteStream, Length};
use std::{
    io::Write,
    path::{Path, PathBuf},
    time::Instant,
};

use aws_sdk_s3::operation::copy_object::{CopyObjectError, CopyObjectOutput};
use flate2::write::GzDecoder;
use uuid::Uuid;
const REGION: &str = "REGION";
const DELAY_TIME: std::time::Duration = std::time::Duration::from_secs(10);

use tokio::time::Duration;

pub type S3ObjectStream = aws_smithy_async::future::pagination_stream::PaginationStream<
    Result<
        aws_sdk_s3::operation::list_objects_v2::ListObjectsV2Output,
        aws_sdk_s3::error::SdkError<
            aws_sdk_s3::operation::list_objects_v2::ListObjectsV2Error,
            aws_smithy_runtime_api::http::Response,
        >,
    >,
>;

pub async fn paginagion_stream_code(
    client: &S3Client,
    bucket: &str,
    prefix: &str,
) -> S3ObjectStream {
    client
        .list_objects_v2()
        .bucket(bucket)
        .prefix(prefix)
        .into_paginator()
        .send()
}

/// Take a local file and upload to a bucket using the provided key
pub async fn upload(
    path: PathBuf,
    bucket_name: &str,
    key: &str,
    // file_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let body = ByteStream::from_path(Path::new(&path)).await;
    // let config = aws_config::from_env().region(REGION).load().await;
    let config = aws_config::defaults(BehaviorVersion::v2023_11_09())
        .region(REGION)
        .load()
        .await;
    println!("key {:?}", key);
    let client = S3Client::new(&config);
    let _ = client
        .put_object()
        .bucket(bucket_name)
        .key(key)
        .body(body.unwrap())
        .send()
        .await;
    println!("Uploaded file: {}", key);
    Ok(())
}
pub async fn show_objects(bucket: &str, prefix: &str) {
    // let config = aws_config::from_env().region(REGION).load().await;
    let config = aws_config::defaults(BehaviorVersion::v2023_11_09())
        .region("REGION")
        .load()
        .await;
    let client = S3Client::new(&config);

    let objects = client
        .list_objects_v2()
        .bucket(bucket)
        .prefix(prefix)
        .into_paginator();

    let mut stream = objects.send();

    while let Ok(Some(chunk)) = stream.try_next().await {
        println!("got a chunk");
        println!("{:?}", chunk)
    }
}

//In bytes, minimum chunk size of 5MB. Increase CHUNK_SIZE to send larger chunks.(I put 100MB)
const CHUNK_SIZE: u64 = 1024 * 1024 * 100;
const MAX_CHUNKS: u64 = 10000;

pub async fn multipart_upload(path: PathBuf, bucket_name: &str, key: &str) -> Result<(), Error> {
    // let config = aws_config::from_env().region(REGION).load().await;
    let config = aws_config::defaults(BehaviorVersion::v2023_11_09())
        .region("REGION")
        .load()
        .await;
    println!("key {:?}", key);
    let client = S3Client::new(&config);

    // snippet-start:[rust.example_code.s3.create_multipart_upload]
    let multipart_upload_res: CreateMultipartUploadOutput = client
        .create_multipart_upload()
        .bucket(bucket_name)
        .key(key)
        .send()
        .await
        .unwrap();
    // snippet-end:[rust.example_code.s3.create_multipart_upload]
    let upload_id = multipart_upload_res.upload_id().unwrap();

    let file_size = tokio::fs::metadata(&path)
        .await
        .expect("it exists I swear")
        .len();

    let mut chunk_count = (file_size / CHUNK_SIZE) + 1;
    let mut size_of_last_chunk = file_size % CHUNK_SIZE;
    if size_of_last_chunk == 0 {
        size_of_last_chunk = CHUNK_SIZE;
        chunk_count -= 1;
    }

    if file_size == 0 {
        panic!("Bad file size.");
    }
    if chunk_count > MAX_CHUNKS {
        panic!("Too many chunks! Try increasing your chunk size.")
    }

    let mut upload_parts: Vec<CompletedPart> = Vec::new();

    for chunk_index in 0..chunk_count {
        let this_chunk = if chunk_count - 1 == chunk_index {
            size_of_last_chunk
        } else {
            CHUNK_SIZE
        };
        let stream = ByteStream::read_from()
            .path(&path)
            .offset(chunk_index * CHUNK_SIZE)
            .length(Length::Exact(this_chunk))
            .build()
            .await
            .unwrap();
        //Chunk index needs to start at 0, but part numbers start at 1.
        let part_number = (chunk_index as i32) + 1;
        // snippet-start:[rust.example_code.s3.upload_part]
        let upload_part_res = client
            .upload_part()
            .key(key)
            .bucket(bucket_name)
            .upload_id(upload_id)
            .body(stream)
            .part_number(part_number)
            .send()
            .await
            .unwrap();
        upload_parts.push(
            CompletedPart::builder()
                .e_tag(upload_part_res.e_tag.unwrap_or_default())
                .part_number(part_number)
                .build(),
        );
        // snippet-end:[rust.example_code.s3.upload_part]
    }
    // snippet-start:[rust.example_code.s3.upload_part.CompletedMultipartUpload]
    let completed_multipart_upload: CompletedMultipartUpload = CompletedMultipartUpload::builder()
        .set_parts(Some(upload_parts))
        .build();
    // snippet-end:[rust.example_code.s3.upload_part.CompletedMultipartUpload]

    // snippet-start:[rust.example_code.s3.complete_multipart_upload]
    let _complete_multipart_upload_res = client
        .complete_multipart_upload()
        .bucket(bucket_name)
        .key(key)
        .multipart_upload(completed_multipart_upload)
        .upload_id(upload_id)
        .send()
        .await
        .unwrap();
    // snippet-end:[rust.example_code.s3.complete_multipart_upload]

    Ok(())
}

pub async fn copy(
    bucket: &str,
    from_key: &str,
    to_key: &str,
    // ) -> Result<CopyObjectOutput, Error> {
) -> Result<CopyObjectOutput, SdkError<CopyObjectError>> {
    let mut source_bucket_and_object: String = "".to_owned();
    source_bucket_and_object.push_str(bucket);
    source_bucket_and_object.push('/');
    source_bucket_and_object.push_str(from_key);
    // let config = aws_config::from_env().region(REGION).load().await;
    let config = aws_config::defaults(BehaviorVersion::v2023_11_09())
        .region("REGION")
        .load()
        .await;
    use aws_sdk_s3::Client;
    let client = Client::new(&config);
    client
        .copy_object()
        .copy_source(source_bucket_and_object)
        .bucket(bucket)
        .key(to_key)
        .send()
        .await
    // .expect("something went wrong with copying");
}

pub async fn download(bucket_name: &str, key: &str) -> Result<Vec<u8>, Error> {
    // key: &str ) -> Result<Vec<u8>, Error> {
    // let config = aws_config::from_env().region(REGION).load().await;
    let config = aws_config::defaults(BehaviorVersion::v2023_11_09())
        .region("REGION")
        .load()
        .await;
    println!("config {:?}", config);
    let s3 = S3Client::new(&config);
    // let data: GetObjectOutput = s3_service::download_object(&client, &bucket_name, &key).await?;

    let result = s3
        .get_object()
        .bucket(bucket_name)
        .key(key)
        .send()
        .await
        .unwrap();
    let body = result.body;

    // use futures::TryStreamExt;
    println!("before data: ");
    let data: Vec<u8> = body
        .collect()
        .await
        .map(|data| data.into_bytes())
        .unwrap()
        .to_vec();

    println!("body_length: {:?}", data.len());
    Ok(data)
}

pub async fn get_body_from_s3(bucket_name: &str, key: &str) -> Result<String, ()> {
    // let config = aws_config::from_env().region(REGION).load().await;
    let config = aws_config::defaults(BehaviorVersion::v2023_11_09())
        .region("REGION")
        .load()
        .await;
    println!("config {:?}", config);
    // use aws_sdk_s3::Client;
    let s3 = S3Client::new(&config);
    let result = s3
        .get_object()
        .bucket(bucket_name)
        .key(key)
        .send()
        .await
        .unwrap();
    let body = result.body;

    let mut streaming_body = body;
    {
        let mut body = Vec::new();
        while let Some(chunk) = streaming_body.next().await {
            body.extend_from_slice(chunk.unwrap().as_ref());
        }
        let mut writer = Vec::new();
        let mut decoder = GzDecoder::new(writer);
        decoder.write_all(&body[..]).unwrap();
        writer = decoder.finish().unwrap();
        let s = String::from_utf8(writer).expect("String parsing error");
        Ok(s)
    }
}

pub async fn get_execution_id(
    query: String,
    catalog: Option<String>,
    database: Option<String>,
) -> String {
    // let config = aws_config::from_env().region(REGION).load().await;
    let config = aws_config::defaults(BehaviorVersion::v2023_11_09())
        .region("REGION")
        .load()
        .await;
    // println!("config {:?}", config);
    let athena = athena_client::new(&config);
    let request_token = Uuid::new_v4();
    eprintln!("{:?}", &request_token);

    let query_execution_context = QueryExecutionContext::builder()
        .set_database(database)
        .set_catalog(catalog)
        .build();
    // .set_database(Some("datascience_parquet".to_string())).set_catalog(Some("AwsDataCatalog".to_string())).build();
    let result_configuration = ResultConfiguration::builder()
        .output_location("s3://aws-athena-query-results-id-REGION/".to_string())
        .set_encryption_configuration(None)
        .set_expected_bucket_owner(None)
        .build();

    let query_input = athena
        .start_query_execution()
        .set_query_string(Some(query))
        // .set_query_string(Some("select id, name from beacons limit 5".to_string()))
        .set_work_group(Some("primary".to_string()))
        .set_query_execution_context(Some(query_execution_context))
        .set_result_configuration(Some(result_configuration))
        .send()
        .await;

    println!("{:?}", &query_input);
    let execution_id = match query_input {
        Ok(output) => match output.query_execution_id {
            Some(query_id) => query_id,
            None => "".to_string(),
        },
        Err(_error) => "".to_string(),
    };
    println!("query running. id: {}", &execution_id);
    // let _query_execution_input = GetQueryExecutionInput {
    //     query_execution_id: Some(execution_id.clone()),
    // };

    let mut interval = tokio::time::interval(Duration::from_secs(10));

    println!("query running. id: {}", &execution_id);

    let athena_call_time = Instant::now();
    let _guard = sentry::init((
        "https://id@sentry.domain.com/9?timeout=10,verify_ssl=0",
        sentry::ClientOptions {
            release: sentry::release_name!(),
            environment: Some("production".into()),
            ..Default::default()
        },
    ));

    loop {
        tokio::select! {
                _ = interval.tick() => {
                    match athena.get_query_execution().set_query_execution_id(Some(execution_id.clone())).send().await {

                                Ok(result) => match result
                                                    .query_execution
                                                    .unwrap()
                                                    .status
                                                    .unwrap()
                                                    .state.unwrap().as_str()
                        {
                                    "SUCCEEDED" => { break;}
        ,
                                    _ => if athena_call_time.elapsed().as_millis() < 600_000 {
                                    // _ => if athena_call_time.elapsed().as_millis() < 10 {
                                            println!("Waiting for the query.");
                                        } else {
                                            let message = "Panic in Athena query: 10 min elapsed for Athena call".to_string();
                                            println!("{}", message);
                                            sentry::capture_message(message.as_str(), sentry::Level::Warning);
                                            break;
                                        },

                        },

                            _ => println!("Unexpected result while calling Athena"),
                            }
                        }
                    }
    }
    println!("execution_id {:?}", &execution_id);
    execution_id
}

/// Take a local file and upload to a bucket using the provided key
pub async fn read_view(
    query: Option<String>,
    catalog: Option<String>,
    database: Option<String>,
    // ) -> GetQueryResultsOutput {
) -> Vec<ResultSet> {
    // let config = aws_config::from_env().region(REGION).load().await;
    let config = aws_config::defaults(BehaviorVersion::v2023_11_09())
        .region("REGION")
        .load()
        .await;
    // println!("config {:?}", config);
    let athena = athena_client::new(&config);

    let request_token = Uuid::new_v4();
    println!("{:?}", &request_token);

    let query_execution_context = QueryExecutionContext::builder()
        .set_database(database)
        .set_catalog(catalog)
        .build();
    let result_configuration = ResultConfiguration::builder()
        .output_location("s3://aws-athena-query-results-id-REGION/".to_string())
        .set_encryption_configuration(None)
        .set_expected_bucket_owner(None)
        .build();

    let query_input = athena
        .start_query_execution()
        // .set_query_string(Some("select id, name from table limit 5".to_string()))
        .set_query_string(query)
        .set_work_group(Some("primary".to_string()))
        .set_query_execution_context(Some(query_execution_context))
        .set_result_configuration(Some(result_configuration))
        .send()
        .await;

    let execution_id = match query_input {
        Ok(output) => output.query_execution_id.unwrap_or_default(),
        Err(_error) => "".to_string(),
    };
    println!("query running. id: {}", &execution_id);

    let mut interval = tokio::time::interval(Duration::from_secs(10));

    let mut next_token: Option<String> = None;
    let count_token = 0;
    let mut results: Vec<ResultSet> = vec![];
    let athena_call_time = Instant::now();
    // pretty_env_logger::init();
    let _guard = sentry::init((
        "https://id@sentry.domain.com/9?timeout=10,verify_ssl=0",
        sentry::ClientOptions {
            release: sentry::release_name!(),
            environment: Some("production".into()),
            ..Default::default()
        },
    ));

    loop {
        tokio::select! {
        _ = interval.tick() => {
                    match athena.get_query_execution().set_query_execution_id(Some(execution_id.clone())).send().await {
                        Ok(result) => match result
                                            .query_execution
                                            .unwrap()
                                            .status
                                            .unwrap()
                                            .state.unwrap().as_str()
                {
                            "SUCCEEDED" =>
                                if count_token == 0 {
                                    match athena.get_query_results().set_query_execution_id(Some(execution_id.clone()))
                    .set_next_token( next_token.clone())
                    .set_max_results(Some(1_000)).send().await
                    {
                                        Ok(output) =>  {
                            println!("output token {:?}", output.next_token) ;
                            next_token = output.next_token.clone();
                            if output.next_token.is_none() {
                                println!("NONE");
                                break;
                            } else {
                                results.extend(output.result_set);

                                  println!("{:?} result_set", results.len());
                                  // println!("{:?} token", results.len());
                                  println!("{:?} athena call time", athena_call_time.elapsed().as_millis());
                            }
                            println!("going again");
                         },
            Err(error) => {
                println!("Error: {:?}", error);
                                             }
        }
                    } else {
                    break;
                },
                            _ => if athena_call_time.elapsed().as_millis() < 600_000 {
                            // _ => if athena_call_time.elapsed().as_millis() < 10 {
                                    println!("Waiting for the query.");
                                } else {
                                    let message = "Panic in Athena query: 10 min elapsed for Athena call".to_string();
                                    println!("{}", message);
                                    sentry::capture_message(message.as_str(), sentry::Level::Warning);
                                    break;
                                },

                },

                    _ => println!("Unexpected result while calling Athena"),
                    }
                }
            }
    }
    results
}

pub async fn create_crawler(
    crawler_name: String,
    path: String,
    _greedy: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let _crawler_targets = path.clone();
    let iam_role = "arn:aws:iam::id:role/service-role/AWSGlueServiceRole-datascience".to_string();
    // let config = aws_config::from_env().region(REGION).load().await;
    let config = aws_config::defaults(BehaviorVersion::v2023_11_09())
        .region("REGION")
        .load()
        .await;
    // let glue = aws_sdk_glue::Client::new(&config);
    use aws_sdk_glue::Client;
    let glue = Client::new(&config);

    let get_crawler = glue
        .get_crawler()
        .name(crawler_name.clone())
        .send()
        .await
        .unwrap();

    let must_create = match get_crawler {
        GetCrawlerOutput {
            crawler: Some(Crawler { name, .. }),
            ..
        } => match name {
            Some(_crawler_name) => false,
            _ => panic!("nothing here"),
        },
        _ => true,
    };

    if must_create {
        let create_crawler = glue
            .create_crawler()
            .name(crawler_name.clone())
            .database_name("datascience_parquet".to_string())
            .role(iam_role)
            .targets(
                CrawlerTargets::builder()
                    .s3_targets(S3Target::builder().path(path).build())
                    .build(),
            )
            .send()
            .await;
        println!("create crawler success {:?}", create_crawler.unwrap())
    } else {
        println!("crawler already exists")
    }

    Ok(())
}

pub async fn update_crawler(
    database_name: String,
    crawler_name: String,
    // schedule: &str,
    path: String,
) -> Result<(), Box<dyn std::error::Error>> {
    // let config = aws_config::from_env().region(REGION).load().await;
    let config = aws_config::defaults(BehaviorVersion::v2023_11_09())
        .region("REGION")
        .load()
        .await;
    use aws_sdk_glue::Client;
    let glue = Client::new(&config);
    println!("{:?} path for crawler", path);
    // let attempts = 0;
    let update_crawler = glue
        .update_crawler()
        .name(crawler_name.clone())
        .database_name(database_name)
        // .schedule(schedule)
        .targets(
            CrawlerTargets::builder()
                .s3_targets(S3Target::builder().path(path).build())
                .build(),
        )
        .send()
        .await;

    match update_crawler {
        Ok(_) => {
            println!("crawler {} has been updated", &crawler_name);
        }
        Err(crawler_error) => match crawler_error {
            SdkError::ServiceError(err) => match err.err() {
                UpdateCrawlerError::CrawlerRunningException(_) => {
                    println!("crawler update failed due to running state. bailing out.");
                }
                UpdateCrawlerError::EntityNotFoundException(_) => {
                    println!("not found")
                }
                UpdateCrawlerError::OperationTimeoutException(_) => {
                    println!("timed out")
                }
                _ => {
                    println!("no idea")
                }
            },
            _ => unimplemented!(
                "don't know what's wrong with an update {:#?}",
                (crawler_error)
            ),
        },
    }
    Ok(())
}

pub async fn get_crawler_state(crawler_name: &str) -> Result<(), Error> {
    // let config = aws_config::from_env().region(REGION).load().await;
    let config = aws_config::defaults(BehaviorVersion::v2023_11_09())
        .region("REGION")
        .load()
        .await;
    use aws_sdk_glue::Client;
    let glue = Client::new(&config);
    let crawler_state = glue
        .get_crawler()
        .name(crawler_name)
        .send()
        .await
        .unwrap()
        .crawler
        .unwrap()
        .state
        .unwrap();
    println!(" state {:?}", &crawler_state);

    Ok(())
}

pub async fn start_crawler(
    crawler_name: String,
    poll_to_completion: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = aws_config::defaults(BehaviorVersion::v2023_11_09())
        .region("REGION")
        .load()
        .await;
    // let config = aws_config::from_env().region(REGION).load().await;

    let glue = aws_sdk_glue::Client::new(&config);
    // use futures::future::err;
    let mut attempts = 0;
    loop {
        let start_crawler = glue.start_crawler().name(crawler_name.clone()).send().await;
        attempts += 1;

        match start_crawler {
            Ok(_) => {
                println!("crawling away on {}", crawler_name);
                break;
            }

            Err(crawler_error) => {
                if let SdkError::ServiceError(err) = crawler_error {
                    match err.err() {
                        StartCrawlerError::CrawlerRunningException(_) => {
                            println!("crawler update failed due to running state. bailing out.");
                            if !poll_to_completion {
                                println!("crawler failed. bailing out.");
                                break;
                            } else {
                                if attempts < 20 {
                                    println!("crawler already running, retrying in 5 seconds")
                                } else {
                                    panic!("crawler has tried 20 times. dying")
                                }
                                std::thread::sleep(DELAY_TIME);
                            }
                        }
                        StartCrawlerError::EntityNotFoundException(_) => {
                            println!("not found")
                        }
                        StartCrawlerError::OperationTimeoutException(_) => {
                            println!("timed out")
                        }
                        _ => {
                            println!("no idea")
                        }
                    }
                }

                if poll_to_completion {
                    wait_for_crawler(&glue, &crawler_name).await?
                }
            }
        }
    }
    Ok(())
}

async fn wait_for_crawler(
    glue: &aws_sdk_glue::client::Client,
    crawler_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let response = glue
            .get_crawler()
            .name(crawler_name)
            .send()
            .await
            .unwrap()
            .crawler;

        match response {
            Some(crawler_resp) => {
                if let Some(state) = crawler_resp.state {
                    if state == ("RUNNING".into()) {
                        println!("crawler is RUNNING, going to sleep {:?}", DELAY_TIME);
                        std::thread::sleep(DELAY_TIME);
                        continue;
                    } else if state == ("STOPPING".into()) {
                        println!(
                            "crawler is stopping... will check for READY in {:?}",
                            DELAY_TIME
                        );
                        std::thread::sleep(DELAY_TIME);
                        continue;
                    } else if state == ("READY".into()) {
                        println!("crawler is done!");
                        break;
                    } else {
                        panic!("weird state, got {:?}", state)
                    }
                } else {
                    panic!("no crawler?!")
                }
            }
            e => panic!("error?! {:#?}", e),
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    // use futures::TryFutureExt;
    // use std::error::Error;
    use std::path::PathBuf;

    // use aws_config::meta::region::RegionProviderChain;
    // use aws_config::Config;
    // use aws_credential_types::provider::future::ProvideCredentials;
    // use aws_credential_types::Credentials;
    use aws_sdk_s3::Client;

    use crate::aws::{upload, REGION};

    // #[test]
    #[tokio::test]
    async fn it_works() {
        let config = aws_config::from_env().region(REGION).load().await;
        println!("{:?} pr", &config.credentials_provider().unwrap());
        let _client = Client::new(&config);
        // use aws_credential_types::provider::ProvideCredentials;
        upload(PathBuf::new(), "domain-datawarehouse", "test/test.txt")
            .await
            .unwrap();
        println!("done");
    }
}
