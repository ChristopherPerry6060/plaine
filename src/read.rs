use crate::plan::Entry;
use anyhow::{anyhow, bail, Error, Result};
use parity_wordlist::random_phrase;
use std::fs::read_dir;
use std::hash::Hasher;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Generate a random id, consisting of two parity words and a uuid.
fn gen_pwid() -> String {
    let word = random_phrase(1);
    let word2 = random_phrase(1);
    let uuid = Uuid::new_v4();
    format!("{word}-{word2}-{uuid}")
}

// Monthly storage fees.
#[derive(serde::Deserialize, Debug, Default, Clone)]
pub struct MonthlyStorageFees {
    #[serde(alias = "asin")]
    asin: Option<String>,
    #[serde(alias = "fnsku")]
    fnsku: String,
    #[serde(alias = "product_name")]
    product_name: Option<String>,
    #[serde(alias = "fulfillment_center")]
    _fulfillment_center: Option<String>,
    #[serde(alias = "country_code")]
    _country_code: Option<String>,
    #[serde(alias = "longest_side")]
    longest_side: Option<f32>,
    #[serde(alias = "median_side")]
    median_side: Option<f32>,
    #[serde(alias = "shortest_side")]
    shortest_side: Option<f32>,
    #[serde(alias = "measurement_units")]
    measurement_units: Option<String>,
    #[serde(alias = "weight")]
    weight: Option<f32>,
    #[serde(alias = "weight_units")]
    weight_units: Option<String>,
    #[serde(alias = "item_volume")]
    item_volume: Option<f32>,
    #[serde(alias = "volume_units")]
    volume_units: Option<String>,
    #[serde(alias = "product_size_tier")]
    product_size_tier: Option<String>,
    #[serde(alias = "average_quantity_on_hand")]
    _average_quantity_on_hand: Option<f32>,
    #[serde(alias = "average_quantity_pending_removal")]
    _average_quantity_pending_removal: Option<f32>,
    #[serde(alias = "estimated_total_item_volume")]
    _estimated_total_item_volume: Option<f32>,
    #[serde(alias = "month_of_charge")]
    _month_of_charge: Option<String>,
    #[serde(alias = "storage_rate")]
    _storage_rate: Option<f32>,
    #[serde(alias = "currency")]
    _currency: Option<String>,
    #[serde(alias = "estimated_monthly_storage_fee")]
    _estimated_monthly_storage_fee: Option<f32>,
    #[serde(alias = "dangerous_goods_storage_type")]
    dangerous_goods_storage_type: Option<String>,
    #[serde(alias = "eligible_for_inventory_discount")]
    _eligible_for_inventory_discount: Option<String>,
    #[serde(alias = "qualifies_for_inventory_discount")]
    _qualifies_for_inventory_discount: Option<String>,
    #[serde(alias = "total_incentive_fee_amount")]
    _total_incentive_fee_amount: Option<String>,
    #[serde(alias = "breakdown_incentive_fee_amount")]
    _breakdown_incentive_fee_amount: Option<String>,
    #[serde(alias = "average_quantity_customer_orders")]
    _average_quantity_customer_orders: Option<String>,
}

impl MonthlyStorageFees {
    fn from_path<P>(path: P) -> Result<Vec<MonthlyStorageFees>, csv::Error>
    where
        P: AsRef<Path>,
    {
        let mut rdr = csv::Reader::from_path(path)?;
        let msf = rdr
            .records()
            .filter_map(|x| x.ok())
            .map(|x| x.deserialize(None))
            .filter_map(|x| x.ok())
            .collect::<Vec<MonthlyStorageFees>>();
        Ok(msf)
    }
}

/// Made from the report that is located [here].
///
/// [here](https://sellercentral.amazon.com/reportcentral/AFNInventoryReport/1).
#[derive(serde::Deserialize, Debug, Default, Clone)]
pub struct AmzFbaInventory {
    #[serde(alias = "sku")]
    msku: String,
    #[serde(alias = "fnsku")]
    fnsku: String,
    #[serde(alias = "asin")]
    asin: String,
    #[serde(alias = "product-name")]
    title: String,
    #[serde(alias = "condition")]
    condition: String,
}
impl AmzFbaInventory {
    fn from_path<P>(path: P) -> Result<Vec<AmzFbaInventory>, csv::Error>
    where
        P: AsRef<Path>,
    {
        let mut rdr = csv::Reader::from_path(path)?;
        let afi = rdr
            .records()
            .filter_map(|x| x.ok())
            .map(|x| x.deserialize(None))
            .filter_map(|x| x.ok())
            .collect::<Vec<AmzFbaInventory>>();
        Ok(afi)
    }
}

/// Mutate `entries` in place through lookup in the `.local` directory.
///
/// See:
/// * [`AmzFbaInventory`]
/// * [`MonthlyStorageFees`]
fn fill_entries(entries: &mut Vec<Entry>) -> Result<(), Error> {
    let afi_vec = read_dir(".local")?
        .filter_map(|x| x.ok())
        .filter_map(|x| AmzFbaInventory::from_path(x.path()).ok())
        .flatten()
        .collect::<Vec<_>>();

    let msf_vec = read_dir(".local")?
        .filter_map(|x| x.ok())
        .filter_map(|x| MonthlyStorageFees::from_path(x.path()).ok())
        .flatten()
        .collect::<Vec<_>>();

    for item in entries {
        // AmzFbaInventory pulling.
        if let Some(found) = afi_vec.iter().find(|row| row.fnsku == item.get_fnsku()) {
            let msku = found.msku.clone();
            let condition = found.condition.clone();
            item.set_msku(Some(msku));
            item.set_condition(Some(condition));
        };

        // MonthlyStorageFees pulling.
        if let Some(found) = msf_vec.iter().find(|row| row.fnsku == item.get_fnsku()) {
            item.set_title(found.product_name.clone());
            item.set_amz_size(found.product_size_tier.clone());
            item.set_asin(found.asin.clone());
            item.set_total_pounds(found.weight);
            item.set_amz_size(found.product_size_tier.clone());

            // These might not be very accurate, so don't overwrite 
            // what we already have.
            let amz_dims = [
                found.longest_side.unwrap_or_default(),
                found.median_side.unwrap_or_default(),
                found.shortest_side.unwrap_or_default(),
            ];
            item.set_amz_dimensions(Some(amz_dims));
        };
    }
    Ok(())
}

/// From the "shipping plans" within Google Drive.
#[derive(Default, serde::Serialize, serde::Deserialize, Debug)]
struct GDriveEntry {
    #[serde(alias = "Info")]
    info: Option<String>,
    #[serde(alias = "FNSKU")]
    fnsku: Option<String>,
    #[serde(alias = "Quantity")]
    quantity: Option<u32>,
    #[serde(alias = "Pack Type")]
    pack_type: Option<String>,
    #[serde(alias = "Staging Group")]
    staging_group: Option<String>,
    #[serde(alias = "Unit Weight")]
    unit_weight: Option<f32>,
    #[serde(alias = "Case QT")]
    case_qt: Option<u32>,
    #[serde(alias = "Print Order")]
    print_order: Option<String>,
    #[serde(alias = "Case Length")]
    case_length: Option<f32>,
    #[serde(alias = "Case Width")]
    case_width: Option<f32>,
    #[serde(alias = "Case Height")]
    case_height: Option<f32>,
    #[serde(alias = "Case Weight")]
    case_weight: Option<f32>,
    #[serde(alias = "Total Weight")]
    total_weight: Option<f32>,
    #[serde(alias = "Total Cases")]
    _total_cases: Option<u32>,
    #[serde(alias = "Readable")]
    _readable: Option<String>,
}

#[derive(Default, Debug)]
struct GDrivePlan {
    helper: Vec<GDriveEntry>,
}
impl GDrivePlan {
    pub fn proc_from_path<P>(path: P) -> Result<Vec<Entry>>
    where
        P: AsRef<Path>,
    {
        let mut rdr = csv::Reader::from_path(path)?;
        let ok_recs = rdr.records().filter_map(|rec| rec.ok());
        let de_recs = ok_recs.filter_map(|unw| unw.deserialize(None).ok());
        let good_recs = de_recs
            .filter(|gdp: &GDriveEntry| gdp.fnsku.is_some())
            .collect::<Vec<GDriveEntry>>();
        let gdp = GDrivePlan { helper: good_recs };
        Vec::<Entry>::try_from(gdp)
    }

    fn _from_path<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
        let mut rdr = csv::Reader::from_path(path).unwrap();
        let ok_recs = rdr.records().filter_map(|rec| rec.ok());
        let de_recs = ok_recs.filter_map(|unw| unw.deserialize(None).ok());
        let good_recs = de_recs
            .filter(|gdp: &GDriveEntry| gdp.fnsku.is_some())
            .collect::<Vec<GDriveEntry>>();
        GDrivePlan { helper: good_recs }
    }
}

impl TryFrom<GDrivePlan> for Vec<Entry> {
    type Error = anyhow::Error;

    fn try_from(value: GDrivePlan) -> std::result::Result<Self, Self::Error> {
        // weird parent / child trailt rules.
        let value0 = value.helper;
        let mut conversions = value0
            .into_iter()
            .filter_map(|x| Vec::<Entry>::try_from(x).ok())
            .flatten()
            .collect::<Vec<_>>();
        match fill_entries(&mut conversions) {
            Ok(_) => dbg!("no issues"),
            Err(_) => dbg!("issues"),
        };
        Ok(conversions)
    }
}
impl TryFrom<GDriveEntry> for Vec<Entry> {
    type Error = anyhow::Error;

    fn try_from(value: GDriveEntry) -> Result<Self, Self::Error> {
        let fnsku = value
            .fnsku
            .clone()
            .ok_or(anyhow!("Expected Fnsku in {value:#?}."))?;
        let units = value
            .quantity
            .ok_or(anyhow!("Expected Fnsku in {value:#?}."))?;

        let mut helper = vec![];
        match &value.pack_type {
            Some(x) if x == "Loose" => {
                let pounds = value.unit_weight.unwrap_or_default();
                let total_pounds = (pounds * units as f32).round();
                let mut entry = Entry::default();
                let id = value.staging_group.unwrap_or_default();
                entry.set_fnsku(fnsku);
                entry.set_units(units as i32);
                entry.set_total_pounds(Some(total_pounds));
                entry.set_id(id);
                helper.push(entry);
            }
            _ => {
                let per_case = value
                    .case_qt
                    .ok_or(anyhow!("Expect 'Case Qt' in {value:#?}."))?;
                if !matches!(units.checked_rem(per_case), Some(0)) {
                    bail!("Expect 'Total Qt' to be evenly divisible by 'Case Qt' in {value:#?}.");
                };
                let cases = units.checked_div(per_case).unwrap_or_default();
                if cases.eq(&0) {
                    bail!("Expected {value:#?} to not be zero cases.");
                };
                let case_weight = value.case_weight;
                for _ in 0..cases {
                    let mut entry = Entry::default();
                    entry.set_fnsku(fnsku.clone());
                    entry.set_units(units as i32);
                    entry.set_id(gen_pwid());
                    entry.set_total_pounds(case_weight);
                    helper.push(entry.clone());
                }
            }
        };
        Ok(helper)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_this() {
        let _o = GDrivePlan::proc_from_path("GdDrivePlan.csv");
    }
}
