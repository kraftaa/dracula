use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use uuid::Uuid;

use super::{Currency, Milestone, Shipping};

#[derive(Queryable, Debug)]
pub struct PurchaseOrder {
    pub id: i32,
    pub version_number: i32,
    pub referenced_po_id: Option<i32>,
    pub ad_po_number: Option<String>,
    pub payment_terms: Option<i32>,
    pub po_number: Option<String>,
    pub po_created_at: Option<NaiveDateTime>,
    pub accepted: Option<bool>,
    pub rejected: Option<bool>,
    pub obsoleted: Option<bool>,
    pub completed: Option<bool>,
    pub justifications: Vec<String>,
    pub commission_rate: Option<BigDecimal>,
    pub provider_shipping_address_id: Option<i32>,
    pub customer_shipping_address_id: Option<i32>,
    pub cpo_id: Option<i32>,
    pub proposal_id: Option<i32>,
    pub created_at: Option<NaiveDateTime>,
    pub uuid: Option<Uuid>,
    pub total_price: BigDecimal,
    pub subtotal_price: BigDecimal,
    pub provider_id: Option<i32>,
    pub order_id: Option<i32>,
    pub type_: Option<String>,
    pub supplier_updated: Option<bool>,
    pub updated_at: Option<NaiveDateTime>,
    pub user_id: Option<i32>,
    pub request_id: Option<i32>,
}

impl PurchaseOrder {
    pub fn status(&self) -> Option<String> {
        match (
            self.rejected,
            self.obsoleted,
            self.updated(),
            self.completed,
            self.purchase_requisition(),
            self.purchase_order(),
        ) {
            (_, Some(true), _, _, _, _) => Some("Obsoleted".to_string()),
            (_, _, true, _, _, _) => Some("Amended".to_string()),
            (_, _, _, Some(true), _, _) => Some("Completed".to_string()),
            (Some(true), _, _, _, _, _) => Some("Rejected".to_string()),
            (_, _, _, _, true, _) => Some("Awaiting PO".to_string()),
            (_, _, _, _, _, true) => Some("Work in Progress".to_string()),
            _ => None,
        }
    }

    pub fn updated(&self) -> bool {
        self.version_number > 1 && self.referenced_po_id.is_some()
    }

    pub fn purchase_requisition(&self) -> bool {
        self.po_number.is_none()
    }

    pub fn purchase_order(&self) -> bool {
        !self.purchase_requisition()
    }

    pub fn retail_subtotal_price(&self, milestones: &[&Milestone]) -> BigDecimal {
        milestones.iter().fold(BigDecimal::from(0), |acc, m| {
            acc + m.total_price.clone()
        })
    }

    pub fn retail_subtotal_tax_amount(&self, milestones: &[&Milestone]) -> BigDecimal {
        milestones.iter().fold(BigDecimal::from(0), |acc, m| {
            acc + m.total_tax_amount()
        })
    }

    pub fn retail_total_price(&self, milestones: &[&Milestone], shipping: &Shipping) -> BigDecimal {
        let retail_subtotal_price = self.retail_subtotal_price(milestones);
        let tax_price = self.retail_subtotal_tax_amount(milestones);
        let shipping_price = shipping.cost.clone();

        retail_subtotal_price + tax_price + shipping_price
    }

    pub fn retail_total_price_usd(
        &self,
        milestones: &[&Milestone],
        shipping: &Shipping,
        currency: &Currency,
    ) -> BigDecimal {
        let default_conversion_rate = BigDecimal::from(1); // TODO move to static?
        let conversion_rate = currency
            .conversion_rate
            .as_ref()
            .unwrap_or(&default_conversion_rate);

        self.retail_total_price(milestones, shipping) / conversion_rate
    }
}
