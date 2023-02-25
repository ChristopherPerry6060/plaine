use std::u8;
use csv::StringRecord;
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
impl AllListingsReport {
    /// Attempt to deserialize from a path.
    ///
    /// # Errors
    ///
    /// * The supplied path is invalid.
    /// * Deserialization fails.
    pub fn from_path<P>(path: P) -> anyhow::Result<Vec<AllListingsReport>>
    where
        P: AsRef<Path>,
    {
        // Set the delimiter to a tab char.
        let mut rdr = csv::ReaderBuilder::new().delimiter(b'\t').from_path(path)?;
        let expect_hdr = StringRecord::from(vec![
            "seller-sku",
            "asin1",
            "item-name",
            "product-id-type",
            "item-condition",
            "product-id",
        ]);

        let hdr = rdr.headers()? ;
        if  hdr != &expect_hdr {
            return Err(anyhow!("Expected{expect_hdr:#?}, got {hdr:#?}"));
        };
        let alr = rdr
            .records()
            .filter_map(|x| x.ok())
            .map(|x| x.deserialize(None))
            .filter_map(|x| x.ok())
            .collect::<Vec<AllListingsReport>>();
        Ok(alr)
    }
}

/// Product conditions for an Asin.
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
