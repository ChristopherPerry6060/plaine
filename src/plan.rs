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
    pub fn set_dimensions(&mut self, dims: [f32; 3]) {
        let v = dims.to_vec();
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
}
