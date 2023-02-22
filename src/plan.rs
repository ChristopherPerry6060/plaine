use std::collections::HashMap;

pub trait Plan {
    fn entries(&self) -> &Vec<Entry>;

    /// Return a `HashMap` that is keyed with box Ids leading to Entries
    fn fold_cases(&self) -> HashMap<String, Vec<&Entry>> {
        self.entries()
            .iter()
            .fold(HashMap::new(), |mut acc, entry| {
                match acc.get_mut(entry.get_id()) {
                    Some(case) => {
                        case.push(entry);
                    }
                    None => {
                        let id = entry.get_id().to_string();
                        acc.insert(id, vec![entry]).expect("New hashmap key");
                    }
                };
                acc
            })
    }

    /// Return the total case count of the [`Plan`].
    fn total_cases(&self) -> usize {
        self.fold_cases().keys().count()
    }

    fn get_as_sums(&self) -> Vec<Entry> {
        self.entries()
            .iter()
            .fold(HashMap::<String, Entry>::new(), |mut acc, x| {
                match acc.get_mut(x.get_fnsku()) {
                    Some(inner_entry) => {
                        inner_entry.units += x.get_units();
                    }
                    None => {
                        acc.insert(x.fnsku.to_owned(), x.clone());
                    }
                };
                acc
            })
            .into_values()
            .collect::<Vec<_>>()
    }

    fn units_of_skus(&self) -> HashMap<String, i32> {
        self.entries()
            .iter()
            .fold(HashMap::new(), |mut acc, entry| {
                match acc.get_mut(entry.get_fnsku()) {
                    Some(units) => {
                        *units += entry.get_units();
                    }
                    None => {
                        let id = entry.get_id().to_string();
                        acc.insert(id, entry.units).expect("New hashmap key");
                    }
                };
                acc
            })
    }
}

impl Plan for Vec<Entry> {
    fn entries(&self) -> &Vec<Entry> {
        self
    }
}

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
