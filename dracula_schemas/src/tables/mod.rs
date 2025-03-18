mod action_items_tl;
pub use self::action_items_tl::*;

mod addresses_tl;
pub use self::addresses_tl::*;

pub mod currencies_tl;
pub use self::currencies_tl::*;

mod invoices_tl;
pub use self::invoices_tl::*;

pub mod proposals_tl;
pub use self::proposals_tl::*;

mod providers_tl;
pub use self::providers_tl::*;

mod purchase_orders_tl;
pub use self::purchase_orders_tl::*;

mod shippings_tl;
pub use self::shippings_tl::*;

mod taxes_tl;
pub use self::taxes_tl::*;

mod users_tl;
pub use self::users_tl::*;

mod organizations_tl;
pub use self::organizations_tl::*;

pub mod orders_tl;
pub use self::orders_tl::*;

mod requests_tl;
pub use self::requests_tl::*;

pub mod milestones_tl;
pub use self::milestones_tl::*;

pub mod notes_tl;
pub use self::notes_tl::*;

pub mod refs_tl;
pub use self::refs_tl::*;

mod wares_tl;
pub use self::wares_tl::*;

mod wpc_tl;
pub use self::wpc_tl::*;

pub mod events_tl;
pub use self::events_tl::*;

pub mod embedded_dynamic_forms_tl;
pub use self::embedded_dynamic_forms_tl::*;

mod inventory_groups_tl;
pub use self::inventory_groups_tl::*;

mod inventories_tl;
pub use self::inventories_tl::*;

mod ratings_tl;
pub use self::ratings_tl::*;

mod turn_around_times_tl;
pub use self::turn_around_times_tl::*;

mod timepoints_tl;
pub use self::timepoints_tl::*;

joinable!(inventories -> inventory_groups (inventory_group_id));
allow_tables_to_appear_in_same_query!(inventory_groups, inventories);
