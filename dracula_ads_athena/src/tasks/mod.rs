mod clicks_stream_task;
pub use self::clicks_stream_task::*;

extern crate regex;
use chrono::{NaiveDateTime, TimeZone, Utc};
use regex::Regex;
pub fn get_naive_datetime_from_value(v: &str) -> Option<NaiveDateTime> {
    // let date_str = v.as_str()?;

    let re = Regex::new(r"/Date\((\d+)\)/").ok()?;
    // let timestamp_str = re.captures(date_str)?.get(1)?.as_str();
    let timestamp_str = re.captures(v)?.get(1)?.as_str();

    let timestamp: i64 = timestamp_str.parse().ok()?;

    // Convert from milliseconds to seconds
    let secs = timestamp / 1000;
    let nano_secs = ((timestamp % 1000) * 1_000_000) as u32;

    Some(
        Utc.timestamp_opt(secs, nano_secs)
            .single()
            .unwrap()
            .naive_utc(),
    )
}
pub mod prelude {
    pub use std::time::Instant;

    pub use diesel::dsl::any;
    pub use diesel::pg::PgConnection;
    pub use diesel::prelude::*;

    pub use bigdecimal::{BigDecimal, ToPrimitive};
    pub use chrono::prelude::*;
    pub use std::collections::HashMap;

    pub use ::function_name::named;
    pub use diesel::result::Error;
    pub use dracula_parquet::prelude::*;
    pub use dracula_parquet::props;
    pub use dracula_parquet::FileWriterRows;
    pub use dracula_schemas::models::*;
    pub use log::*;
    pub use parquet::file::properties::WriterProperties;
    pub use rayon::prelude::*;
    pub use regex::Regex;
    pub use std::convert::TryFrom;
    pub use std::fs;
    pub use std::fs::File;
    pub use std::sync::Arc;

    pub use std::path::Path;
}
