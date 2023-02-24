use super::*;

/// Representation of the All Listings Report (Custom).
///
/// Compatible with the standard report of the same name.
#[derive(Default, Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)]
pub(super) struct AllListingsReport {
    #[serde(rename(deserialize = "seller-sku"))]
    pub(super) seller_sku: Option<String>,
    #[serde(rename(deserialize = "asin1"))]
    pub(super) asin: Option<String>,
    #[serde(rename(deserialize = "item-name"))]
    pub(super) item_name: Option<String>,
    #[serde(rename(deserialize = "product-id-type"))]
    pub(super) product_id_type: Option<String>,
    #[serde(rename(deserialize = "item-condition"))]
    #[serde(deserialize_with = "translate_condition")]
    pub(super) item_condition: Condition,
    #[serde(rename(deserialize = "product-id"))]
    pub(super) product_id: Option<String>,
}

/// Product condtions that an Asin can be sold as.
#[derive(PartialEq, Default, Debug)]
pub(super) enum Condition {
    #[default]
    New,
    UsedLikeNew,
    None,
}

// Custom deserialization for Amz's coding of item conditions.
//
// * `11`: [`Condition::New`].
// * `1`: [`Condition::UsedLikeNew`].
//
// Other variants exist within the reports but these are the ones we
// tend to care about.
fn translate_condition<'de, D>(deserializer: D) -> Result<Condition, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let c = u32::deserialize(deserializer)?;
    if c == 11 {
        Ok(Condition::New)
    } else if c == 1 {
        Ok(Condition::UsedLikeNew)
    } else {
        Ok(Condition::None)
    }
}
