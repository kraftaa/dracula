// Order matters!
extern crate openssl;

extern crate diesel;

use dracula_tasks::tasks::DraculaStreamingTask;
use dracula_tasks::tasks::DraculaTask;
use dracula_tasks::tasks::HugeTask;

#[derive(Debug, serde::Deserialize)]
#[allow(non_snake_case)]
pub struct Args {
    pub arg_POSTGRES_URI: String,
    pub flag_table: String,
    pub flag_limit: usize,
    pub flag_upload: String,
    pub flag_file: String,
    pub flag_ads: String,
}
// condenced
pub const USAGE: &str = "
Dracula

Usage:
  dracula (<POSTGRES-URI>) [--table=<table>] [--upload=<S3_URL>] [--file=<file>] [--ads=<file>]

Options:
  -l --limit=<LIMIT>    Number of documents per request [default: 1000]
  -h --help             Show this screen.
  -t --table=<TABLE>    Postgres table to process
  -u --upload=<S3_URL>  Target file [default: s3://default-datawarehouse]
  -n --no-upload        Skip uploading to S3
  -b --file=<file>      Shows to use booster
  -k --ads=<file>     Shows to run ads data load
";

pub const DATABASE: &str = "datascience_parquet";
pub const DATABASE_STREAM: &str = "datascience_streaming_test";

pub const CHANNEL: &str = "#Rust";
pub const USERNAME: &str = "Dracula";
pub const EMOJI: &str = ":funny-bat:";
pub const BASE_PATH: &str = "default-datawarehouse";
pub const MAIN_FOLDER: &str = "parquet-files";
pub const MAIN_FOLDER_STREAM: &str = "parquet_files_streaming_test";
pub const CRAWLER_NAME: &str = "mighty_dracula";
pub const CRAWLER_NAME_ONE: &str = "mighty_dracula_oneoff";
pub const ADS_CRAWLER: &str = "ads-tasks";
pub const ADS_FOLDER: &str = "ads";
pub const ADS_DATABASE: &str = "ads";

pub fn event_tasks_list() -> Vec<(&'static str, Box<dyn HugeTask>)> {
    let event_tasks: Vec<(&str, Box<dyn HugeTask>)> =
        vec![("events", Box::new(dracula_tasks::tasks::EventTask {}))];
    event_tasks
}

pub fn events_emails_tasks_list() -> Vec<(&'static str, Box<dyn HugeTask>)> {
    let event_tasks: Vec<(&str, Box<dyn HugeTask>)> = vec![(
        "events_emails",
        Box::new(dracula_tasks::tasks::EventEmailTask {}),
    )];
    event_tasks
}

pub fn inventories_list() -> Vec<(&'static str, Box<dyn DraculaStreamingTask>)> {
    let inventories_tasks: Vec<(&str, Box<dyn DraculaStreamingTask>)> = vec![(
        "inventories",
        Box::new(dracula_tasks::tasks::InventoriesStTask {}),
    )];
    inventories_tasks
}

pub fn streaming_tasks_list() -> Vec<(&'static str, Box<dyn DraculaStreamingTask>)> {
    let streaming_tasks: Vec<(&str, Box<dyn DraculaStreamingTask>)> = vec![
        (
            "notes",
            Box::new(dracula_tasks::tasks::NoteStreamingTask {}),
        ),
        (
            "refs_providers",
            Box::new(dracula_tasks::tasks::RefsPrStreamingTask {}),
        ),
        ("wpcs", Box::new(dracula_tasks::tasks::WPCStreamingTask {})),
    ];
    streaming_tasks
}

pub fn tasks_list() -> Vec<(&'static str, Box<dyn DraculaTask>)> {
    let tasks: Vec<(&str, Box<dyn DraculaTask>)> = vec![
        ("addresses", Box::new(dracula_tasks::tasks::AddressTask {})),
        (
            "currencies",
            Box::new(dracula_tasks::tasks::CurrencyTask {}),
        ),
        (
            "inventory_groups",
            Box::new(dracula_tasks::tasks::InventoryGroupTask {}),
        ),
        ("invoices", Box::new(dracula_tasks::tasks::InvoiceTask {})),
        (
            "organizations",
            Box::new(dracula_tasks::tasks::OrganizationTask {}),
        ),
        (
            "proposals",
            Box::new(dracula_tasks::tasks::ProposalPageTask {}),
        ),
        ("providers", Box::new(dracula_tasks::tasks::ProviderTask {})),
        (
            "purchase_orders",
            Box::new(dracula_tasks::tasks::PurchaseOrderTask {}),
        ),
        ("requests", Box::new(dracula_tasks::tasks::RequestTask {})),
        ("orders", Box::new(dracula_tasks::tasks::OrderTask {})),
        (
            "refs_users",
            Box::new(dracula_tasks::tasks::RefsUserTaskPart {}),
        ),
        ("shippings", Box::new(dracula_tasks::tasks::ShippingTask {})),
        ("taxes", Box::new(dracula_tasks::tasks::TaxTask {})),
        ("users", Box::new(dracula_tasks::tasks::UserTask {})),
        ("wares", Box::new(dracula_tasks::tasks::WareTask {})),
    ];
    tasks
}

pub fn ads_streaming_tasks_list() -> Vec<(&'static str, Box<dyn DraculaStreamingTask>)> {
    let ads_streaming_tasks: Vec<(&str, Box<dyn DraculaStreamingTask>)> = vec![(
        "clicks_stream_task",
        Box::new(dracula_ads_athena::tasks::ClickStreamTask {}),
    )];
    ads_streaming_tasks
}

pub fn embedded_dynamic_form_tasks_list() -> Vec<(&'static str, Box<dyn HugeTask>)> {
    let embedded_dynamic_form_tasks: Vec<(&str, Box<dyn HugeTask>)> = vec![(
        "embedded_dynamic_forms",
        Box::new(dracula_tasks::tasks::EmbeddedDynamicFormTask {}),
    )];
    embedded_dynamic_form_tasks
}
