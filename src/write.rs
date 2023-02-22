use crate::plan::{Entry, Plan};
use std::path::PathBuf;

pub fn write_check_file(entry_vec: Vec<Entry>, plan_name: String) -> std::io::Result<()> {
    let mut contents = String::new();
    let name = format!("{plan_name}\n");

    let hdr = "ASIN,TITLE,UNITS,SIZE,FNSKU,UPC,COUNT,NOTES\n".to_string();
    contents.push_str(&name);
    contents.push_str(&hdr);
    for entry in entry_vec.get_as_sums() {
        // Is the fnsku in the selected set?

        let amz_size = entry.get_amz_size().clone().unwrap_or_default();
        let fnsku = entry.get_fnsku();
        let asin = entry.get_asin().clone().unwrap_or_default();
        let title = entry.get_title().clone().unwrap_or_default();
        let units = entry.get_units().to_string();
        let row = format!("\"{asin}\",\"{title}\",\"{units}\",\"{amz_size}\",\"{fnsku}\",'\n");
        contents.push_str(&row);
    }
    let path = PathBuf::from(format!("{plan_name}-CheckFile.csv"));
    std::fs::write(path, contents)
}
