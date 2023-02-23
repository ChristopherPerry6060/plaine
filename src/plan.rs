use std::collections::HashMap;

pub trait Plan {
    fn entries(&self) -> Vec<Entry>;

    /// Return the number of cases in [`Self`] with more than 0 units.
    fn number_of_real_cases(&self) -> usize {
        // Flatten would help here.
        self.as_folded_cases()
            .into_iter()
            .map(|(_, v)| v.into_iter().map(|x: Entry| x.units).sum::<i32>())
            .filter(|x| x.is_positive())
            .count()
    }

    /// Return the number of cases in [`Self`] with amounts not equal to 0.
    fn number_of_nonzero_cases(&self) -> usize {
        // Flatten would help here.
        self.as_folded_cases()
            .into_iter()
            .map(|(_, v)| v.into_iter().map(|x: Entry| x.units).sum())
            .filter(|x: &i32| x != &0)
            .count()
    }

    /// Return the number of cases in [`Self`] with amounts that are below 0.
    fn negative_unit_case_count(&self) -> usize {
        // Flatten would help here.
        self.as_folded_cases()
            .into_iter()
            .map(|(_, v)| v.into_iter().map(|x: Entry| x.units).sum())
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
    /// want to use [`Self::positive_units_case_count`].
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
                let id = entry.get_id().to_string();
                acc.insert(id, entry.units).expect("New hashmap key");
                acc
            }
        };

        // Fold each entry with equal skus into each other.
        self.entries().into_iter().fold(HashMap::new(), fold)
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
    dimensions: Option<[f32; 3]>,
    amz_dimensions: Option<[f32; 3]>,
}
impl Entry {
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

    pub fn get_units(&self) -> &i32 {
        &self.units
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
            self.dimensions = None;

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
        self.dimensions = Some([l as f32, w as f32, h as f32]);
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
}

