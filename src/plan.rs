pub mod status;

use crate::{Brn, Fnsku, TreeJson};
use anyhow::Result;
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};
use uuid::Uuid;

pub trait Plan {
    fn entries(&self) -> Vec<Entry>;

    /// Returns a clone of [`Self`] without the provided [`Fnsku`]s.
    ///
    /// When [`Self`] does not contain any of the provided Fnsku, this is
    /// just returning [`Self`] cloned and unchanged.
    ///
    /// [Fnksu]: (plaine::Fnsku)
    fn remove_fnskus<I>(&self, i: I) -> Vec<Entry>
    where
        I: IntoIterator<Item = Fnsku>,
    {
        let pre = i.into_iter().collect::<HashSet<_>>();
        self.entries()
            .into_iter()
            // i is list of items to remove
            .filter(|x| !pre.contains(x.get_fnsku()))
            .collect()
    }

    /// Serialize [`Self`] and write the result to the given `path`.
    ///
    /// Given a reference to `self`, serialize to Json format. Write the
    /// file to the given `path`, naming it with the `trunk`, and optionally
    /// appending a `branch`. Lastly, a uuid is pushed to the file name before
    /// writing. `trunk` is in the form of a `&str`, as is the inner `branch`.
    ///
    /// The generated uuid is returned as a confirmation that the write did
    /// not fail.
    ///
    /// # Errors
    ///
    /// This function will not force a write to the file system in any way.
    /// If the given `path` cannot be written to, this will return an error.
    ///
    /// Additionally, serialization can fail prior to a write occurring, this
    /// will return an error as well.
    fn serialize_and_write<P>(&self, branch: Brn, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let uuid = Uuid::new_v4();
        let p = path.as_ref().display();
        let file_name = format!("{p}{branch}_{uuid}.json");
        let json = self.serialize()?;
        Ok(std::fs::write(file_name, json)?)
    }

    /// Serialize [`Self`] into Json format.
    ///
    /// This function will return the full [`TreeJson`] and does not
    /// write anything to the filesystem.
    ///
    /// See [Self::serialize_to_fs] for a version that writes to the local
    /// directory.
    fn serialize(&self) -> Result<TreeJson> {
        let value = self.entries();
        let json = serde_json::to_string_pretty(&value)?;
        Ok(json)
    }

    fn as_negated(&self) -> Vec<Entry> {
        self.entries()
            .into_iter()
            .map(|mut x| {
                let old_units = x.get_units();
                let new_units = old_units.checked_neg().unwrap_or_default();
                x.set_units(new_units);
                x
            })
            .collect()
    }

    /// Return the number of cases in [`Self`] with more than 0 units.
    fn number_of_real_cases(&self) -> usize {
        self.as_folded_cases()
            .into_values()
            .map(|v| v.into_iter().map(|x: Entry| x.units).sum::<i32>())
            .filter(|x| x.is_positive())
            .count()
    }

    /// Return the number of cases in [`Self`] with amounts not equal to 0.
    fn number_of_nonzero_cases(&self) -> usize {
        self.as_folded_cases()
            .into_values()
            .map(|v| v.into_iter().map(|x: Entry| x.units).sum())
            .filter(|x: &i32| x != &0)
            .count()
    }

    /// Return the number of cases in [`Self`] with amounts that are below 0.
    fn negative_unit_case_count(&self) -> usize {
        self.as_folded_cases()
            .into_values()
            .map(|v| v.into_iter().map(|x: Entry| x.units).sum())
            .filter(|x: &i32| x != &0)
            .count()
    }

    /// Returns a copy of [`Self`], mapped by case id, and summed by equal fnsku.
    fn as_folded_cases(&self) -> HashMap<String, Vec<Entry>> {
        let grouped = self.as_group_by_case();
        grouped.iter().fold(HashMap::new(), |mut acc, (k, v)| {
            let vv = v.to_owned().get_as_sums();
            acc.insert(k.to_string(), vv);
            acc
        })
    }

    /// Returns a copy of [`Self`] as raw entries, mapped by case id.
    fn as_group_by_case(&self) -> HashMap<String, Vec<Entry>> {
        let hm = HashMap::new();
        self.entries()
            .iter()
            .fold(hm, |mut acc, entry| match acc.get_mut(entry.get_id()) {
                Some(case) => {
                    case.push(entry.clone());
                    acc
                }
                None => {
                    let id = entry.get_id().to_string();
                    acc.insert(id, vec![entry.clone()]);
                    acc
                }
            })
    }

    /// Return the total cases that [`Self`] has seen.
    ///
    /// As the internal of [`Self`] is akin to a ledger, it may have
    /// records that are currently negated. This function will still
    /// count those cases.
    ///
    /// This function is mostly used for internal records. You probably
    /// want to use [`Self::number_of_real_cases`].
    fn number_of_seen_cases(&self) -> usize {
        self.as_group_by_case().keys().count()
    }

    /// Sum all Entries of [`Self`] into like Fnskus.
    ///
    /// Note that this function breaks the definition of [`Entry`].
    /// Each instance of an entry is bounded by two conditions.
    /// * It cannot span more than one physical box.
    /// * It must adjustments of a single sku.
    ///
    /// This function differs from the others within [`Self`] in that it
    /// immediately breaks the first bound.
    fn get_as_sums(&self) -> Vec<Entry> {
        let fold = |mut acc: HashMap<String, Entry>, x: Entry| {
            if let Some(inner_entry) = acc.get_mut(x.get_fnsku()) {
                inner_entry.units += x.get_units();
                acc
            } else {
                acc.insert(x.fnsku.to_owned(), x.clone());
                acc
            }
        };
        let hashmap = HashMap::<String, Entry>::new();
        let iter = self.entries().into_iter();
        // Fold each eq Fnsku into itself.
        // No need to keep the keys, Entry has a sku field.
        iter.fold(hashmap, fold).into_values().collect()
    }

    /// Return a HashMap containing fnsku as a key, and units as value.
    fn units_of_skus(&self) -> HashMap<String, i32> {
        let fold = |mut acc: HashMap<String, i32>, entry: Entry| {
            // Skus that have already been seen can be added to.
            if let Some(units) = acc.get_mut(entry.get_fnsku()) {
                *units += entry.get_units();
                acc
            } else {
                // New skus can be inserted
                let id = entry.get_fnsku().to_string();
                acc.insert(id, entry.units);
                acc
            }
        };

        // Fold each entry with equal skus into each other.
        self.entries().into_iter().fold(HashMap::new(), fold)
    }

    /// Returns the sum of all units in [`Self`].
    fn units(&self) -> i32 {
        self.entries()
            .get_as_sums()
            .into_iter()
            .map(|x| x.get_units())
            .sum()
    }

    fn get_case_named(&self, case_name: &str) -> Vec<Entry> {
        self.entries()
            .into_iter()
            .filter(|x| x.get_id().eq(case_name))
            .collect::<Vec<_>>()
    }
}

impl Plan for Vec<Entry> {
    fn entries(&self) -> Vec<Entry> {
        self.clone()
    }
}

/// Each instance of an entry is bounded by two conditions.
///
/// * [`Self`] cannot span more than one physical box.
/// * [`Self`] always describes a single Sku.
///
#[derive(Default, Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct Entry {
    amz_size: Option<String>,
    fnsku: String,
    msku: Option<String>,
    title: Option<String>,
    asin: Option<String>,
    condition: Option<String>,
    units: i32,
    total_pounds: Option<f32>,
    id: String,
    upc: Option<String>,
    case_dimensions: Option<[f32; 3]>,
    amz_dimensions: Option<[f32; 3]>,
}
impl Entry {
    pub fn get_amz_dimensions(&self) -> Option<[f32; 3]> {
        self.amz_dimensions
    }
    pub fn get_case_dimensions(&self) -> Option<[f32; 3]> {
        self.case_dimensions
    }

    pub fn set_amz_size(&mut self, set: Option<String>) {
        self.amz_size = set;
    }

    pub fn set_fnsku(&mut self, set: String) {
        self.fnsku = set;
    }

    pub fn set_msku(&mut self, set: Option<String>) {
        self.msku = set;
    }

    pub fn set_title(&mut self, set: Option<String>) {
        self.title = set;
    }

    pub fn set_asin(&mut self, set: Option<String>) {
        self.asin = set;
    }

    pub fn set_condition(&mut self, set: Option<String>) {
        self.condition = set;
    }

    pub fn set_units(&mut self, set: i32) {
        self.units = set;
    }

    pub fn set_total_pounds(&mut self, set: Option<f32>) {
        self.total_pounds = set;
    }

    pub fn set_id(&mut self, set: String) {
        self.id = set;
    }

    pub fn set_upc(&mut self, set: Option<String>) {
        self.upc = set;
    }

    pub fn get_amz_size(&self) -> &Option<String> {
        &self.amz_size
    }

    pub fn get_fnsku(&self) -> &str {
        &self.fnsku
    }

    pub fn get_msku(&self) -> &Option<String> {
        &self.msku
    }

    pub fn get_title(&self) -> &Option<String> {
        &self.title
    }

    pub fn get_asin(&self) -> &Option<String> {
        &self.asin
    }

    pub fn get_condition(&self) -> &Option<String> {
        &self.condition
    }

    pub fn get_units(&self) -> i32 {
        self.units
    }

    pub fn get_total_pounds(&self) -> &Option<f32> {
        &self.total_pounds
    }

    pub fn get_id(&self) -> &str {
        &self.id
    }

    pub fn get_upc(&self) -> &Option<String> {
        &self.upc
    }

    pub fn set_dimensions(&mut self, dims: Option<[f32; 3]>) {
        let Some(udims) = dims else {
            self.case_dimensions = None;

            return;
        };
        let v = udims.to_vec();
        let mut rounded = v.into_iter().map(|x| x.ceil() as u32).collect::<Vec<_>>();
        rounded.sort();
        let l = rounded
            .pop()
            .expect("vec made from [f32;3] can be popped 3 times.");
        let w = rounded
            .pop()
            .expect("vec made from [f32;3] can be popped 3 times.");
        let h = rounded
            .pop()
            .expect("vec made from [f32;3] can be popped 3 times.");
        self.case_dimensions = Some([l as f32, w as f32, h as f32]);
    }

    pub fn set_amz_dimensions(&mut self, dims: Option<[f32; 3]>) {
        let Some(udims) = dims else {
            self.amz_dimensions = None;

            return;
        };
        let v = udims.to_vec();
        let mut rounded = v.into_iter().map(|x| x.ceil() as u32).collect::<Vec<_>>();
        rounded.sort();
        let l = rounded
            .pop()
            .expect("vec made from [f32;3] can be popped 3 times.");
        let w = rounded
            .pop()
            .expect("vec made from [f32;3] can be popped 3 times.");
        let h = rounded
            .pop()
            .expect("vec made from [f32;3] can be popped 3 times.");
        self.amz_dimensions = Some([l as f32, w as f32, h as f32]);
    }

    pub fn str_amz_size(&self) -> &str {
        match &self.amz_size {
            Some(x) => x,
            None => "",
        }
    }

    pub fn str_fnsku(&self) -> &str {
        &self.fnsku
    }

    pub fn str_msku(&self) -> &str {
        match &self.msku {
            Some(x) => x,
            None => "",
        }
    }

    pub fn str_title(&self) -> &str {
        match &self.title {
            Some(x) => x,
            None => "",
        }
    }

    pub fn str_asin(&self) -> &str {
        match &self.asin {
            Some(x) => x,
            None => "",
        }
    }

    pub fn str_condition(&self) -> &str {
        match &self.condition {
            Some(x) => x,
            None => "",
        }
    }

    pub fn str_id(&self) -> &str {
        &self.id
    }

    pub fn str_upc(&self) -> &str {
        match &self.upc {
            Some(x) => x,
            None => "",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_group_by_case() {
        let mut entry1 = Entry::default();

        let id = String::from("ABCD");
        let fnsku = String::from("XDFJHII");

        entry1.set_id(id);
        entry1.set_fnsku(fnsku);
        entry1.set_units(12);

        let mut entry2 = entry1.clone();
        entry2.set_units(-8);

        let plan = vec![entry1, entry2];

        let cases = plan.as_group_by_case();
        assert_eq!(cases.keys().count(), 1);
    }

    #[test]
    fn plan_as_folded_cases() {
        let mut entry1 = Entry::default();
        let id = String::from("Box-id-33");
        let fnsku = String::from("XAEET");

        entry1.set_id(id);
        entry1.set_fnsku(fnsku);
        entry1.set_units(12);

        let mut entry2 = entry1.clone();
        entry2.set_units(15);

        let plan = vec![entry1, entry2];
        let mut folded = plan.as_folded_cases();

        let value = folded
            .remove("Box-id-33")
            .unwrap_or_default()
            .pop()
            .unwrap_or_default();

        assert_eq!(value.get_units(), 27);
    }

    #[test]
    fn plan_case_counts() {
        let mut entry1 = Entry::default();
        let id = String::from("abc");
        let fnsku = String::from("zzz");

        entry1.set_id(id);
        entry1.set_fnsku(fnsku);
        entry1.set_units(12);

        let mut entry2 = entry1.clone();
        entry2.set_units(-12);

        let plan = vec![entry1, entry2];

        let seen = plan.number_of_seen_cases();
        assert_eq!(seen, 1);

        let zero = plan.number_of_nonzero_cases();
        assert_eq!(zero, 0);
    }
    #[test]
    fn negate_in_place() {
        let mut entry1 = Entry::default();
        let id = String::from("abc");
        let fnsku = String::from("zzz");

        entry1.set_id(id);
        entry1.set_fnsku(fnsku);
        entry1.set_units(12);

        let mut entry2 = entry1.clone();
        entry2.set_units(20);

        let plan = vec![entry1, entry2];
        let _neg = plan.as_negated();
    }
    #[test]
    fn serialize() {
        let mut entry1 = Entry::default();
        let id = String::from("abc");
        let fnsku = String::from("zzz");

        entry1.set_id(id);
        entry1.set_fnsku(fnsku);
        entry1.set_units(12);

        let mut entry2 = entry1.clone();
        entry2.set_units(20);
    }
}
