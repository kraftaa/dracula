mod invoices;
pub use self::invoices::*;

mod providers;
pub use self::providers::*;

mod purchase_orders;
pub use self::purchase_orders::*;

mod organizations;
pub use self::organizations::*;

mod users;
pub use self::users::*;

mod addresses;
pub use self::addresses::*;

mod ratings;
pub use self::ratings::*;

mod currencies;
pub use self::currencies::*;

pub mod orders;
pub use self::orders::*;

mod requests;
pub use self::requests::*;

mod notes;
pub use self::notes::*;

mod page_iter;
pub use self::page_iter::*;

mod refs_providers;
pub use self::refs_providers::*;

mod refs_users;
pub use self::refs_users::*;
mod wares;
pub use self::wares::*;

mod wpcs;
pub use self::wpcs::*;

mod events;
pub use self::events::*;

mod proposals;
pub use self::proposals::*;

mod taxes;
pub use self::taxes::*;

mod shippings;
pub use self::shippings::*;

use async_trait::async_trait;
use chrono::NaiveDate;
use std::fmt::Debug;
pub use std::panic::catch_unwind;
pub use std::panic::RefUnwindSafe;
pub use std::panic::UnwindSafe;
use std::path::PathBuf;

pub trait DraculaTask: Sync + Send + RefUnwindSafe + UnwindSafe {
    fn run(&self, postgres_uri: &str) -> (String, i64);
}

#[async_trait]
pub trait DraculaStreamingTask: Debug + Sync + Send + RefUnwindSafe + UnwindSafe {
    async fn run(&self, postgres_uri: &str) -> (String, i64);
}

pub trait HugeTask: Sync + Send + RefUnwindSafe + UnwindSafe {
    fn run(&self, postgres_uri: &str) -> Vec<(NaiveDate, PathBuf, u128, i64)>;
}

pub mod prelude {
    pub use std::time::Instant;

    pub use diesel::dsl::any;
    pub use diesel::pg::PgConnection;
    pub use diesel::prelude::*;

    pub use bigdecimal::{BigDecimal, ToPrimitive};
    pub use chrono::prelude::*;
    //    pub use crate::aws::*;
    //    pub use crate::parquet::prelude::*;
    pub use std::collections::HashMap;

    pub use rayon::prelude::*;
    // TODO: Is this too big of a module include? Does it slow down the compiler?
    pub use super::DraculaStreamingTask;
    pub use super::DraculaTask;
    pub use super::HugeTask;
    pub use ::function_name::named;
    pub use diesel::result::Error;
    pub use dracula_parquet::prelude::*;
    pub use dracula_parquet::props;
    pub use dracula_parquet::FileWriterRows;
    pub use dracula_schemas::models::*;
    pub use log::*;
    pub use parquet::file::properties::WriterProperties;
    pub use std::convert::TryFrom;
    pub use std::fs;
    pub use std::fs::File;
    pub use std::sync::Arc;

    pub use std::path::Path;
}

mod embedded_dynamic_forms;
pub use self::embedded_dynamic_forms::*;

mod milestones;
pub use self::milestones::*;

mod inventory_groups;
pub use self::inventory_groups::*;

mod inventories;
pub use self::inventories::*;

mod events_emails;
pub use self::events_emails::*;
